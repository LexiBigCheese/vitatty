use std::ffi::CString;

use derive_more::{From, Into};

#[derive(Debug, Clone)]
pub enum ShaderError {
    NoShader,
    String(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::NoShader => write!(f, "No Shader"),
            ShaderError::String(s) => write!(f, "Shader did not compile:\n{}", s),
        }
    }
}

impl std::error::Error for ShaderError {}

#[derive(From, Into, Clone, Copy, PartialEq, Eq)]
pub struct Shader(gl::types::GLuint);

impl Shader {
    pub unsafe fn get_iv(&self, param: gl::types::GLenum) -> i32 {
        let mut var = 0;
        unsafe { gl::GetShaderiv(self.0, param, &mut var) };
        var
    }
    pub unsafe fn get_info_log(&self) -> String {
        let info_len = unsafe { self.get_iv(gl::INFO_LOG_LENGTH) };
        let mut info_log = vec![0u8; info_len as usize];
        unsafe {
            gl::GetShaderInfoLog(
                self.0,
                info_len,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as _,
            )
        };
        unsafe { String::from_utf8_unchecked(info_log) }
    }
    pub unsafe fn delete(&self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}

pub fn load_shader(source: &str, typ: gl::types::GLenum) -> Result<Shader, ShaderError> {
    println!("Compiling Shader:\n{}", source);
    let shader = unsafe { gl::CreateShader(typ) };
    if shader == 0 {
        return Err(ShaderError::NoShader);
    }
    unsafe {
        let source_len = source.len() as i32;
        gl::ShaderSource(shader, 1, &(source.as_ptr() as _), &source_len);
        gl::CompileShader(shader);
        let shader = Shader::from(shader);
        let compiled = shader.get_iv(gl::COMPILE_STATUS);
        if compiled == 0 {
            let info_log = shader.get_info_log();
            shader.delete();
            return Err(ShaderError::String(info_log));
        }
        Ok(shader)
    }
}
