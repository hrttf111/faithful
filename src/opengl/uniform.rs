use std::rc::Rc;
use std::cell::RefCell;

use cgmath::{Matrix4, Matrix, Vector4};

use crate::opengl::gl::{GlCtx, gl_error_panic};

/******************************************************************************/

pub type GlUniform1Cell<V> = Rc<RefCell<GlUniform1<V>>>;

pub trait GlUniform {
    fn bind(&self, gl: &GlCtx, index: i32);
}

pub struct GlUniform1<V> {
    val: V,
}

impl<T> GlUniform1<T> {
    pub fn new(val: T) -> GlUniform1<T> {
        GlUniform1{val}
    }

    pub fn new_rc(val: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(val)))
    }

    pub fn set(&mut self, val: T) {
        self.val = val;
    }

    pub fn get(&self) -> &T {
        &self.val
    }
}

impl GlUniform for GlUniform1<i32> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.Uniform1i(index, self.val);
            gl_error_panic(gl, &format!("Uniform1i {:?}, {:?}", index, self.val));
        }
    }
}

impl GlUniform for GlUniform1<f32> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.Uniform1f(index, self.val);
            gl_error_panic(gl, &format!("Uniform1f {:?}, {:?}", index, self.val));
        }
    }
}

impl GlUniform for GlUniform1<Vector4<f32>> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.Uniform4f(index, self.val.x, self.val.y, self.val.z, self.val.w);
            gl_error_panic(gl, &format!("Uniform4f {:?}", index));
        }
    }
}

impl GlUniform for GlUniform1<Matrix4<f32>> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.UniformMatrix4fv(index, 1, 0/*GL_FALSE*/, self.val.as_ptr());
            gl_error_panic(gl, &format!("UniformMatrix4fv {:?}", index));
        }
    }
}

impl GlUniform for GlUniform1<Vec<u32>> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.Uniform1uiv(index, self.val.len() as i32, self.val.as_ptr());
            gl_error_panic(gl, &format!("Uniform1iv {:?}", index));
        }
    }
}

/******************************************************************************/

