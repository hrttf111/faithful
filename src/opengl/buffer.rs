use std::result::Result;

use std::ptr::null;
use std::ffi::c_void;
use std::os::raw::c_uint;

use gl46::*;

use crate::opengl::gl::{GlCtx, gl_get_error, gl_error_panic};

/******************************************************************************/

pub trait GlBuffer {
    fn bind(&self);
}

pub struct GlBufferStatic {
    gl: GlCtx,
    handle: c_uint,
    size: usize,
    buffer_type: GLenum,
}

pub fn unbind_buffer(gl: &GlCtx, buffer_type: GLenum) {
    unsafe {
        gl.BindBuffer(buffer_type, 0);
    }
}

impl GlBufferStatic {
    pub fn new(gl: &GlCtx, size: usize, buffer_type: GLenum) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.CreateBuffers(1, &mut handle);
            if handle == 0 {
                return Err("Cannot create buffer handle".to_owned())
            }
        }
        let mut buffer = Self{gl: gl.clone(), handle, size: 0, buffer_type};
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
            gl.BindBuffer(self.buffer_type, self.handle);
            gl.BufferSubData(self.buffer_type, offset as isize, size as isize, data);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Buffer sub data error = {:?} - {error:?}", self.handle))
            }
        }
        Ok(())
    }

    pub fn unbind(&self) {
        let gl = &self.gl;
        unsafe {
            gl.BindBuffer(self.buffer_type, 0);
        }
    }
}

impl Drop for GlBufferStatic {
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

impl GlBuffer for GlBufferStatic {
    fn bind(&self) {
        let gl = &self.gl;
        unsafe {
            gl.BindBuffer(self.buffer_type, self.handle);
            gl_error_panic(gl, "Cannot bind buffer");
        }
    }
}
