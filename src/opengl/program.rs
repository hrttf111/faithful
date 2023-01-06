use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use std::fs::File;

use std::io;
use std::io::{Error, ErrorKind};
use std::io::Read;

use std::ptr::{null, null_mut};
use std::ffi::{CString, CStr, c_void};
use std::os::raw::{c_uint, c_int};

use num_traits::Num;
use num_traits::cast::{ToPrimitive, NumCast};

use gl46::*;

use crate::opengl::gl::{GlCtx, gl_get_error};
use crate::opengl::uniform::GlUniform;

/******************************************************************************/

fn c_str_to_cstr<T>(c_str: &[u8], len: T) -> String
where
    T: Num + ToPrimitive
{
    match <usize as NumCast>::from(len) {
        None => "".to_string(),
        Some(0) => "".to_string(),
        Some(i) => unsafe {
            let (c_str_, _n) = c_str.split_at(i);
            CStr::from_bytes_with_nul_unchecked(c_str_).to_string_lossy().into_owned()
        }
    }
}

fn get_log<F>(gl: &GlCtx, handle: c_uint, max_size: usize, f: F) -> Result<String, String>
    where F: Fn(c_uint, i32, *mut c_int, *mut u8) {
    let mut log_data = vec![0u8; max_size];
    let mut log_len = 0;
    f(handle, log_data.len() as i32, &mut log_len, log_data.as_mut_ptr());
    if let Err(error) = gl_get_error(gl) {
        return Err(format!("Cannot get log {error:?}"));
    }
    Ok(c_str_to_cstr(&log_data, log_len))
}

fn get_shader_log(gl: &GlCtx, shader: c_uint) -> Result<String, String> {
    let max_size = get_shader_param(gl, shader, GL_INFO_LOG_LENGTH)?;
    if max_size < 0 {
        return Err(format!("Max size of shader log is negative {max_size:?}"));
    }
    get_log(gl, shader, max_size as usize, |a, b, c, d| unsafe { gl.GetShaderInfoLog(a, b, c, d) } )
}

pub fn get_shader_param(gl: &GlCtx, shader: c_uint, param: GLenum) -> Result<i32, String> {
    let mut val: c_int = 0;
    unsafe {
        gl.GetShaderiv(shader, param, &mut val);
        if let Err(error) = gl_get_error(gl) {
            return Err(format!("Cannot get shader param {param:?} for {shader:?}: {error:?}"));
        }
    }
    Ok(val as i32)
}

/******************************************************************************/

pub trait ShaderLoader {
    fn load(&self, gl: &GlCtx, shader: c_uint, data: Vec<u8>) -> Result<(), String>;
}

pub struct GlShaderLoaderBinary {}

impl ShaderLoader for GlShaderLoaderBinary {
    fn load(&self, gl: &GlCtx, shader: c_uint, data: Vec<u8>) -> Result<(), String> {
        unsafe {
            let mut bl = data.into_boxed_slice();
            let shaders = [shader];
            gl.ShaderBinary(shaders.len() as i32
                            , &shaders as *const c_uint
                            , GL_SHADER_BINARY_FORMAT_SPIR_V
                            , bl.as_mut_ptr() as *mut c_void
                            , bl.len() as i32);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Cannot make shader binary for {shader:?}: {error:?}"));
            }
            let name = CString::new("main").unwrap().into_raw() as *mut u8;
            gl.SpecializeShader(shader, name, 0, null(), null());
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Cannot specialize shader binary for {shader:?}: {error:?}"));
            }
            let success = get_shader_param(gl, shader, GL_COMPILE_STATUS)?;
            if success != 1 {
                let s = get_shader_log(gl, shader)?;
                return Err(format!("Cannot compile shader: {s:?}"));
            }
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub struct GlShader {
    name: String,
    shader: c_uint,
}

impl GlShader {
    pub fn from_file<L: ShaderLoader>(name: &str, path: &str, gl: &GlCtx, loader: &L, shader_type: GLenum) -> io::Result<GlShader> {
        let mut file = File::options().read(true).open(path)?;
        let mut vec = Vec::new();
        file.read_to_end(&mut vec)?;
        let shader = gl.CreateShader(shader_type);
        if shader == 0 {
            if let Err(error) = gl_get_error(gl) {
                return Err(Error::new(ErrorKind::Other, format!("Cannot create shader {name:?}: {error:?}")))
            } else {
                return Err(Error::new(ErrorKind::Other, "Cannot create shader {name:?}: no gl error"))
            }
        }
        if let Err(error) = loader.load(gl, shader, vec) {
            return Err(Error::new(ErrorKind::Other, error));
        }
        Ok(GlShader { name : name.to_string(), shader })
    }

    pub fn attach_from_file<L: ShaderLoader>(name: &str, path: &str, program: &mut GlProgram, loader: &L, shader_type: GLenum) {
        let gl = &program.gl;
        match GlShader::from_file(name, path, gl, loader, shader_type) {
            Err(e) =>
                panic!("Cannot load shader {name:?}({path:?}) -> {e:?}"),
            Ok(s) =>
                if let Err(e) = program.attach_shader(s) {
                    panic!("Cannot load shader {name:?}({path:?}) -> {e:?}");
                }
        }
    }
}

/******************************************************************************/

pub struct GlEntity {
    index: u32,
    gl_type: GLenum,
    name: String,
    size: c_int,
}

impl GlEntity {
    pub fn new(index: u32, gl_type: GLenum, name: String, size: c_int) -> Self {
        Self{index, gl_type, name, size}
    }
}

impl fmt::Display for GlEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {:?}, {}, {})", self.index, self.gl_type, self.name, self.size)
    }
}

#[allow(dead_code)]
pub struct ProgramInfo {
    uniforms: Vec<GlEntity>,
    attrs: Vec<GlEntity>,
    shaders: u16,
}

impl fmt::Display for ProgramInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Uniforms({}):", self.uniforms.len())?;
        for uniform in &self.uniforms {
            writeln!(f, "{}", uniform)?;
        }
        writeln!(f, "Attrs({}):", self.attrs.len())?;
        for attr in &self.attrs {
            writeln!(f, "{}", attr)?;
        }
        writeln!(f, "shaders: {}", self.shaders)
    }
}

struct UniformHolder {
    uniform: Rc<RefCell<dyn GlUniform>>,
    index: i32,
}

pub struct GlProgram {
    gl: GlCtx,
    handle: c_uint,
    shaders: Vec<GlShader>,
    uniforms: Vec<UniformHolder>,
}

impl GlProgram {
    pub fn new(gl: &GlCtx) -> GlProgram {
        let handle = gl.CreateProgram();
        if handle == 0 {
            panic!("Cannot create program");
        }
        GlProgram{gl: gl.clone(), handle, shaders: Vec::new(), uniforms: Vec::new()}
    }

    pub fn use_program(&self) {
        let gl = &self.gl;
        gl.UseProgram(self.handle);
        self.bind_uniforms();
    }

    pub fn bind_uniforms(&self) {
        let gl = &self.gl;
        for holder in &self.uniforms {
            holder.uniform.borrow().bind(gl, holder.index);
        }
    }

    pub fn attach_shader(&mut self, shader: GlShader) -> Result<(), String> {
        self.attach_shaders(vec![shader])
    }

    pub fn attach_shaders(&mut self, mut shaders: Vec<GlShader>) -> Result<(), String> {
        let gl = &self.gl;
        for shader in &shaders {
            //todo: use glGetAttachedShaders
            gl.AttachShader(self.handle, shader.shader);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Cannot attach shader {:?}: {:?}", shader.name, error));
            }
        }
        gl.LinkProgram(self.handle);
        if let Err(error) = gl_get_error(gl) {
            return Err(format!("Cannot link program {error:?}"));
        }

        let success = self.get_program_value(GL_LINK_STATUS)?;
        if success != 1 {
            //detach?
            return Err("Cannot link program".to_string())
        }
        self.shaders.append(&mut shaders);
        Ok(())
    }

    pub fn get_uniform_location(&self, s: &str) -> Result<u32, String> {
        let gl = &self.gl;
        let v: c_int;
        unsafe {
            let sc = CString::new(s).unwrap();
            v = gl.GetUniformLocation(self.handle, sc.as_ptr() as *const u8);
            if let Err(error) = gl_get_error(gl) {
                return Err(format!("Cannot get uniform location of {s:?}: {error:?}"));
            }
            if v < 0 {
                return Err(format!("Uniform location of {s:?} is negative {v:?}"));
            }
        }
        Ok(v as u32)
    }

    pub fn set_uniform(&mut self, index: i32, uniform: Rc<RefCell<dyn GlUniform>>) {
        let holder = UniformHolder{uniform, index};
        self.uniforms.push(holder);
    }

    pub fn set_uniform_name(&mut self, name: &str, uniform: Rc<RefCell<dyn GlUniform>>) {
        let index = self.get_uniform_location(name).unwrap();
        println!("Index = {index:?}");
        self.set_uniform(index as i32, uniform)
    }

    pub fn get_log(&self) -> Result<String, String> {
        let gl = &self.gl;
        let max_size = self.get_program_value(GL_INFO_LOG_LENGTH)?;
        if max_size < 0 {
            return Err(format!("Max size of program log is negative {max_size:?}"));
        }
        get_log(gl, self.handle, max_size as usize, |a, b, c, d| unsafe { gl.GetProgramInfoLog(a, b, c, d) })
    }

    pub fn get_program_interface_value(&self, prog_interface: GLenum, pname: GLenum) -> Result<i32, String> {
        let gl = &self.gl;
        unsafe {
            let mut val: c_int = 0;
            gl.GetProgramInterfaceiv(self.handle, prog_interface, pname, &mut val);
            if let Err(error) = gl_get_error(gl) {
                Err(format!("GetProgramInterfaceiv failed for {prog_interface:?}/{pname:?} with error {error:?}"))
            } else {
                Ok(val as i32)
            }
        }
    }

    pub fn get_program_value(&self, attr: GLenum) -> Result<i32, String> {
        let gl = &self.gl;
        unsafe {
            let mut val: c_int = 0;
            gl.GetProgramiv(self.handle, attr, &mut val);
            if let Err(error) = gl_get_error(gl) {
                Err(format!("GetProgramiv failed for {attr:?} with error {error:?}"))
            } else {
                Ok(val as i32)
            }
        }
    }

    pub fn get_info(&self) -> Result<ProgramInfo, String> {
        let gl = &self.gl;
        unsafe {
            let properties = [GL_BLOCK_INDEX, GL_TYPE, GL_NAME_LENGTH, GL_LOCATION, GL_OFFSET, GL_ARRAY_SIZE];

            let num_uniforms = self.get_program_interface_value(GL_UNIFORM, GL_ACTIVE_RESOURCES)?;
            let max_name_len = self.get_program_interface_value(GL_UNIFORM, GL_MAX_NAME_LENGTH)?;

            if max_name_len < 0 {
                return Err(format!("GL_MAX_NAME_LENGTH is negative {max_name_len:?}"));
            }

            let mut uniforms = Vec::new();
            let mut name = vec![0u8; max_name_len as usize];
            for i in 0..num_uniforms {
                let mut values: [c_int; 6] = [0, 0, 0, 0, 0, 0];
                gl.GetProgramResourceiv(self.handle
                                        , GL_UNIFORM
                                        , i as u32
                                        , values.len() as i32
                                        , properties.as_ptr()
                                        , values.len() as i32
                                        , null_mut()
                                        , values.as_mut_ptr());
                if values[0] != -1 {
                    continue;
                }

                let mut length: c_int = 0;
                gl.GetProgramResourceName(self.handle
                                          , GL_UNIFORM
                                          , i as u32
                                          , name.len() as i32
                                          , &mut length
                                          , name.as_mut_ptr());
                if let Err(error) = gl_get_error(gl) {
                    return Err(format!("GetProgramResourceName failed {error:?}"));
                }
                let entity = GlEntity::new(values[3] as u32, gl46::GLenum(values[1] as u32), c_str_to_cstr(&name, length), values[5]);
                uniforms.push(entity);
            }

            let active_attrs = self.get_program_value(GL_ACTIVE_ATTRIBUTES)?;
            let max_attr_len = self.get_program_value(GL_ACTIVE_ATTRIBUTE_MAX_LENGTH)?;

            if max_attr_len < 0 {
                return Err(format!("GL_ACTIVE_ATTRIBUTE_MAX_LENGTH is negative {max_attr_len:?}"));
            }

            let mut name = vec![0u8; max_attr_len as usize];
            let mut attrs = Vec::new();
            for i in 0..active_attrs {
                let mut length: c_int = 0;
                let mut size: c_int = 0;
                let mut gl_type: GLenum = gl46::GLenum(0);
                gl.GetActiveAttrib(self.handle
                                   , i as u32
                                   , name.len() as i32
                                   , &mut length
                                   , &mut size
                                   , &mut gl_type
                                   , name.as_mut_ptr());
                attrs.push(GlEntity::new(i as u32, gl_type, c_str_to_cstr(&name, length), size));
            }

            let shaders = self.get_program_value(GL_ATTACHED_SHADERS)?;
            Ok(ProgramInfo{uniforms, attrs, shaders: shaders as u16})
        }
    }
}
