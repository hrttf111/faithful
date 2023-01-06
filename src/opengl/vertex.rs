use std::vec::Vec;
use std::result::Result;

use std::ptr::null;
use std::ffi::c_void;
use std::os::raw::c_uint;

use gl46::*;

use crate::opengl::gl::{GlCtx, gl_get_error, gl_error_panic};

/******************************************************************************/

pub struct GlVertexAttr {
    index: c_uint,
    elems: usize,
    elem_type: GLenum,
    offset: usize,
}

impl GlVertexAttr {
    pub fn new(index: c_uint, elems: usize, elem_type: GLenum, offset: usize) -> Self {
        Self{index, elems, elem_type, offset}
    }
}

pub struct GlVao {
    gl: GlCtx,
    handle: c_uint,
    attrs: Vec<GlVertexAttr>,
}

impl GlVao {
    pub fn new(gl: &GlCtx) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut handle);
        }
        Ok(Self{gl: gl.clone(), handle, attrs: Vec::new()})
    }

    pub fn new_bind(gl: &GlCtx, attrs: Vec<GlVertexAttr>) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut handle);
        }
        let vao = Self{gl: gl.clone(), handle, attrs};
        vao.set_attrs_internal()?;
        Ok(vao)
    }

    pub fn bind(&self) {
        let gl = &self.gl;
        gl.BindVertexArray(self.handle);
        gl_error_panic(gl, &format!("Cannot bind VAO {:?}", self.handle));
    }

    pub fn unbind(&self) {
        let gl = &self.gl;
        gl.BindVertexArray(0);
    }

    pub fn set_attrs(&mut self, attrs: Vec<GlVertexAttr>) -> Result<(), String> {
        self.attrs = attrs;
        self.set_attrs_internal()
    }

    fn set_attrs_internal(&self) -> Result<(), String> {
        let gl = &self.gl;
        for attr in &self.attrs {
            unsafe {
                gl.EnableVertexAttribArray(attr.index);
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("EnableVertexAttribArray error = {error:?}"))
                }
                if attr.offset > 0 {
                    gl.VertexAttribPointer(
                        attr.index, // attribute
                        attr.elems as i32,        // number of elements per vertex
                        attr.elem_type,    // the type of each element
                        0 /*GL_FALSE*/,    // take our values as-is
                        0,                 // stride
                        attr.offset as *const c_void
                    );
                } else {
                    gl.VertexAttribPointer(
                        attr.index, // attribute
                        attr.elems as i32,        // number of elements per vertex
                        attr.elem_type,    // the type of each element
                        0 /*GL_FALSE*/,    // take our values as-is
                        0,                 // stride
                        null()
                    );
                }
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("VertexAttribPointer error = {error:?}"))
                }
            }
        }
        Ok(())
    }
}

impl Drop for GlVao {
    fn drop(&mut self) {
        let gl = &self.gl;
        let handle = self.handle;
        unsafe {
            gl.DeleteVertexArrays(1, &handle);
        }
        self.handle = 0;
    }
}
