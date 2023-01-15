use std::result::Result;

use std::ffi::c_void;
use std::os::raw::c_uint;

use gl46::*;

use crate::opengl::gl::{GlCtx, gl_error_panic, gl_clear_error, gl_get_error};

/******************************************************************************/

pub struct GlSampler {
    gl: GlCtx,
    handle: c_uint,
}

pub struct GlBuffer {
    gl: GlCtx,
    handle: c_uint,
}

impl Drop for GlBuffer {
    fn drop(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.DeleteBuffers(1, &self.handle);
        }
        self.handle = 0;
    }
}

impl GlBuffer {
    pub fn new(gl: &GlCtx) -> Self {
        let mut handle: c_uint = 0;
        unsafe {
            gl.CreateBuffers(1, &mut handle);
            gl.BindBuffer(GL_TEXTURE_BUFFER, handle);
            gl_error_panic(gl, "BindBuffer");
        }
        Self{gl: gl.clone(), handle}
    }

    pub fn set_data<V>(&mut self, size: usize, data: &[V]) {
        let gl = &self.gl;
        unsafe {
            let ptr = data.as_ptr() as *const c_void;
            gl.NamedBufferData(self.handle, size as isize, ptr, GL_STATIC_DRAW);
            gl_error_panic(gl, "NamedBufferStorage");
        }
    }
}

pub struct GlTexture {
    gl: GlCtx,
    handle: c_uint,
    tex_type: GLenum,
    uniform: Option<c_uint>,
    buffer: Option<GlBuffer>,
    params: TextureParams,
    width: usize,
    height: usize,
}

impl GlSampler {
    pub fn new(gl: &GlCtx) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.GenSamplers(1, &mut handle);
        }
        let texture = Self{gl: gl.clone(), handle};
        Ok(texture)
    }
}

impl Drop for GlSampler {
    fn drop(&mut self) {
        let gl = &self.gl;
        let handle = self.handle;
        unsafe {
            gl.DeleteSamplers(1, &handle);
        }
        self.handle = 0;
    }
}

#[derive(Copy, Clone)]
pub struct TextureParams {
    pub target: GLenum,
    pub internal_format: GLenum,
    pub format: GLenum,
    pub data_type: GLenum,
    pub nearest: bool,
}

impl GlTexture {
    pub fn new(gl: &GlCtx, tex_type: GLenum, uniform: Option<c_uint>, buffer: Option<GlBuffer>, params: &TextureParams, width: usize, height: usize) -> Result<Self, String> {
        let mut handle: c_uint = 0;
        unsafe {
            gl.CreateTextures(tex_type, 1, &mut handle);
        }
        let texture = Self{gl: gl.clone(), handle, tex_type, uniform, buffer, params: *params, width, height};
        Ok(texture)
    }

    pub fn new_1d<V>(gl: &GlCtx, uniform: Option<c_uint>, params: &TextureParams, width: usize, data: &[V]) -> Result<Self, String> {
        let texture = Self::new(gl, params.target, uniform, None, params, width, 0)?;
        let width = width as i32;
        texture.bind();
        unsafe {
            gl.TextureStorage1D(texture.handle, 1, params.internal_format, width);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("TextureStorage1D error = {error:?} {width:?}"))
            }
            let ptr = data.as_ptr() as *const c_void;
            gl.TextureSubImage1D(texture.handle, 0, 0, width, params.format, params.data_type, ptr);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("TextureSubImage1D error = {error:?}"))
            }
            if params.nearest {
                let nearest = GL_NEAREST.0 as i32;
                gl.TextureParameteriv(texture.handle, GL_TEXTURE_MIN_FILTER, &nearest);
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("TextureParameteriv MIN error = {error:?}"))
                }
                gl.TextureParameteriv(texture.handle, GL_TEXTURE_MAG_FILTER, &nearest);
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("TextureParameteriv MAG error = {error:?}"))
                }
            }
        }
        Ok(texture)
    }

    pub fn new_2d<V>(gl: &GlCtx, uniform: Option<c_uint>, params: &TextureParams, width: usize, height: usize, data: &[V]) -> Result<Self, String> {
        let texture = Self::new(gl, params.target, uniform, None, params, width, height)?;
        let width = width as i32;
        let height = height as i32;
        texture.bind();
        unsafe {
            gl.TextureStorage2D(texture.handle, 1, params.internal_format, width, height);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("TextureStorage2D error = {error:?}"))
            }
            let ptr = data.as_ptr() as *const c_void;
            gl.TextureSubImage2D(texture.handle, 0, 0, 0, width, height, params.format, params.data_type, ptr);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("TextureSubImage2D error = {error:?}"))
            }
            if params.nearest {
                let nearest = GL_NEAREST.0 as i32;
                gl.TextureParameteriv(texture.handle, GL_TEXTURE_MIN_FILTER, &nearest);
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("TextureParameteriv MIN error = {error:?}"))
                }
                gl.TextureParameteriv(texture.handle, GL_TEXTURE_MAG_FILTER, &nearest);
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("TextureParameteriv MAG error = {error:?}"))
                }
            }
        }
        Ok(texture)
    }

    pub fn new_buffered<V>(gl: &GlCtx, uniform: Option<c_uint>, internal_format: GLenum, size: usize, data: &[V]) -> Result<Self, String> {
        let buffer = {
            let mut buffer = GlBuffer::new(gl);
            buffer.set_data(size, data);
            Some(buffer)
        };
        let params = TextureParams{target: GL_TEXTURE_BUFFER, internal_format, format: GL_RGBA_INTEGER, data_type: GL_UNSIGNED_BYTE, nearest: true};
        let texture = Self::new(gl, GL_TEXTURE_BUFFER, uniform, buffer, &params, size, 0)?;
        texture.bind();
        unsafe {
            let handle = match &texture.buffer {
                Some(b) => b.handle,
                None => panic!("No handle"),
            };
            gl.TextureBuffer(texture.handle, internal_format, handle); 
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("TextureBuffer error = {error:?}"))
            }
        }
        Ok(texture)
    }

    pub fn set_data<V>(&mut self, data: &[V]) {
        let gl = &self.gl;
        unsafe {
            if let Some(b) = &mut self.buffer {
                b.set_data(self.width, data);
                return;
            } else if self.width != 0 && self.height != 0 {
                let ptr = data.as_ptr() as *const c_void;
                gl.TextureSubImage2D(self.handle, 0, 0, 0, self.width as i32, self.height as i32, self.params.format, self.params.data_type, ptr);
            } else if self.width != 0 {
                let ptr = data.as_ptr() as *const c_void;
                gl.TextureSubImage1D(self.handle, 0, 0, self.width as i32, self.params.format, self.params.data_type, ptr);
            } else {
                panic!("Unknown texture format");
            }
            gl_error_panic(gl, "TextureSubImage2D");
        }
    }

    pub fn bind(&self) {
        let gl = &self.gl;
        unsafe {
            match self.uniform {
                Some(n) => {
                    gl_clear_error(gl);
                    gl.BindTextureUnit(n, self.handle);
                    gl_error_panic(gl, &format!("BindTextureUnit = {:?}", self.handle));
                },
                None => {
                    gl_clear_error(gl);
                    gl.BindTexture(self.tex_type, self.handle);
                    gl_error_panic(gl, &format!("BindTexture = {:?}", self.handle));
                }
            }
        }
    }
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.DeleteTextures(1, &self.handle);
        }
        self.handle = 0;
    }
}
