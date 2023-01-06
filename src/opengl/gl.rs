use std::rc::Rc;
use std::ffi::CStr;

use gl46::*;
use glutin::PossiblyCurrent;

/******************************************************************************/

pub type GlCtx = Rc<GlFns>;

pub fn new_gl_ctx(gl_context: &glutin::Context<PossiblyCurrent>) -> GlCtx {
    unsafe {
        match GlFns::load_from(&|symbol| gl_context.get_proc_address( &CStr::from_ptr(symbol as *mut i8).to_string_lossy() )) {
            Err(e) =>
                panic!("Cannot init GL {e:?}"),
            Ok(gl) => {
                Rc::new(gl)
            }
        }
    }
}

pub fn gl_error_panic(gl: &GlCtx, text: &str) {
    if let Err(s) = gl_get_error(gl) { panic!("{:?} - {:?}", s, text) }
}

pub fn gl_get_error(gl: &GlCtx) -> Result<(), String> {
    unsafe {
        let error = gl.GetError();
        if error != GL_NO_ERROR {
            Err(format!("GL error {:?}", error))
        } else {
            Ok(())
        }
    }
}

pub fn gl_clear_error(gl: &GlCtx) {
    unsafe {
        gl.GetError();
    }
}

/******************************************************************************/
