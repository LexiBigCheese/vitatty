use derive_more::{From, Into};

use crate::shader::Shader;

#[derive(Debug, Clone)]
pub enum ProgramError {
    NoProgram,
    String(String),
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::NoProgram => write!(f, "No Program"),
            ProgramError::String(s) => write!(f, "Program did not link:\n{}", s),
        }
    }
}

impl std::error::Error for ProgramError {}

#[derive(From, Into, Clone, Copy, PartialEq, Eq, Default)]
pub struct Program(gl::types::GLuint);

impl Program {
    pub unsafe fn get_iv(&self, param: gl::types::GLenum) -> i32 {
        let mut var = 0;
        unsafe { gl::GetProgramiv(self.0, param, &mut var) };
        var
    }
    pub unsafe fn get_info_log(&self) -> String {
        let info_len = unsafe { self.get_iv(gl::INFO_LOG_LENGTH) };
        let mut info_log = vec![0u8; info_len as usize];
        unsafe {
            gl::GetProgramInfoLog(
                self.0,
                info_len,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as _,
            )
        };
        unsafe { String::from_utf8_unchecked(info_log) }
    }
    pub unsafe fn delete(&self) {
        unsafe { gl::DeleteProgram(self.0) }
    }
    pub fn get_attrib_location(&self, attrib: &str) -> i32 {
        unsafe {
            gl::GetAttribLocation(
                self.0,
                std::ffi::CString::new(attrib)
                    .expect("What the hell")
                    .as_ptr() as _,
            )
        }
    }
    pub fn get_uniform_location(&self, uniform: &str) -> i32 {
        unsafe {
            gl::GetUniformLocation(
                self.0,
                std::ffi::CString::new(uniform)
                    .expect("What the hell")
                    .as_ptr() as _,
            )
        }
    }
}

pub fn load_program(vert: Shader, frag: Shader) -> Result<Program, ProgramError> {
    println!("Linking Program:");
    let program = unsafe { gl::CreateProgram() };
    if program == 0 {
        return Err(ProgramError::NoProgram);
    }
    unsafe {
        gl::AttachShader(program, vert.into());
        gl::AttachShader(program, frag.into());
        gl::LinkProgram(program);
        let program = Program::from(program);
        let linked = program.get_iv(gl::LINK_STATUS);
        if linked == 0 {
            let info_log = program.get_info_log();
            program.delete();
            return Err(ProgramError::String(info_log));
        }
        Ok(program)
    }
}
