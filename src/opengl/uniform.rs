use std::rc::Rc;
use std::cell::RefCell;

use std::ptr::null;
use std::ffi::c_void;
use std::os::raw::c_uint;

use cgmath::{Matrix4, Matrix, Vector4};

use gl46::*;

use crate::opengl::gl::{GlCtx, gl_error_panic, gl_get_error};

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

impl GlUniform for GlUniform1<Vector4<i32>> {
    fn bind(&self, gl: &GlCtx, index: i32) {
        unsafe {
            gl.Uniform4i(index, self.val.x, self.val.y, self.val.z, self.val.w);
            gl_error_panic(gl, &format!("Uniform4i {:?}", index));
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

pub struct GlShaderStorage {
    gl: GlCtx,
    handle: c_uint,
    index: c_uint,
    size: usize,
}

impl Drop for GlShaderStorage {
    fn drop(&mut self) {
        let gl = &self.gl;
        let handle = self.handle;
        unsafe {
            gl.DeleteBuffers(1, &handle);
        }
        self.handle = 0;
        self.size = 0;
    }
}

impl GlShaderStorage {
    pub fn new(gl: &GlCtx, size: usize, index: usize) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.CreateBuffers(1, &mut handle);
            if handle == 0 {
                return Err("Cannot create buffer handle".to_owned())
            }
        }
        let mut buffer = Self{gl: gl.clone(), handle, index: index as c_uint, size: 0};
        if size > 0 {
            buffer.alloc(size)?;
        }
        Ok(buffer)
    }

    pub fn alloc(&mut self, size: usize) -> Result<(), String> {
        let gl = &self.gl;
        unsafe {
            gl.NamedBufferStorage(self.handle, size as isize, null(), GL_DYNAMIC_STORAGE_BIT);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Buffer error = {:?} - {error:?}", self.handle))
            }
        }
        self.size = size;
        Ok(())
    }

    pub fn update<T>(&mut self, offset: usize, data: &[T]) -> Result<(), String> {
        self.update_raw(offset, data.len() * std::mem::size_of::<T>(), data.as_ptr() as *const c_void)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn update_raw(&mut self, offset: usize, size: usize, data: *const c_void) -> Result<(), String> {
        if (size + offset) > self.size {
            return Err(format!("Too big update {:?} > {:?}", (size + offset), self.size))
        }
        let gl = &self.gl;
        unsafe {
            gl.BindBufferBase(GL_SHADER_STORAGE_BUFFER, self.index, self.handle);
            gl.BufferSubData(GL_SHADER_STORAGE_BUFFER, offset as isize, size as isize, data);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Buffer sub data error = {:?} - {error:?}", self.handle))
            }
            gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, 0);
        }
        Ok(())
    }

    pub fn unbind(&self) {
        let gl = &self.gl;
        unsafe {
            gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, 0);
        }
    }
}

/******************************************************************************/
