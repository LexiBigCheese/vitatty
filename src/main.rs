pub mod sh_program;
pub mod shader;
pub mod tty_render;

use std::ffi::CString;

// #[link(name = "SDL2", kind = "static")]
#[link(name = "vitaGL", kind = "static")]
#[link(name = "vitashark", kind = "static")]
#[link(name = "SceShaccCg_stub", kind = "static")]
#[link(name = "mathneon", kind = "static")]
#[link(name = "SceShaccCgExt", kind = "static")]
#[link(name = "taihen_stub", kind = "static")]
#[link(name = "SceKernelDmacMgr_stub", kind = "static")]
#[link(name = "SceIme_stub", kind = "static")]
#[link(name = "SceGxm_stub", kind = "static")]
#[link(name = "SceDisplay_stub", kind = "static")]
#[link(name = "SceAppMgr_stub", kind = "static")]
#[link(name = "SceCommonDialog_stub", kind = "static")]
unsafe extern "C" {
    pub fn vglSwapBuffers(has_commondialog: u8);
    pub fn vglSetupRuntimeShaderCompiler(
        opt_level: i32,
        use_fastmath: i32,
        use_fastprecision: i32,
        use_fastint: i32,
    );
    pub fn vglInitExtended(
        legacy_pool_size: i32,
        width: i32,
        height: i32,
        ram_threshold: i32,
        msaa: u32,
    ) -> u8;
    pub fn vglGetTexDataPointer(target: u32) -> *mut u8;
    pub fn vglFree(addr: *mut u8);
    pub fn vglTexImageDepthBuffer(target: u32);
    pub fn vglGetProcAddress(name: *const u8) -> *const u8;
    pub fn vglRemapTexPtr() -> *mut u8;
    pub fn glTexImage2Drgba5(width: i32, height: i32);
    pub fn vglBindFragUbo(index: u32);
}

pub const VERTICES: &'static [f32] = &[0.0f32, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];

unsafe fn setup_gl() {
    unsafe {
        vglSetupRuntimeShaderCompiler(2, 1, 0, 1);
        vglInitExtended(0, 960, 544, 65 * 1024 * 1024, 0);
    }
    gl::load_with(|name| {
        let name = CString::new(name).unwrap();
        unsafe { vglGetProcAddress(name.as_ptr() as _) as _ }
    });
}

fn main_but_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("---- RUN START ----");
    let shaders = [include_str!("tty16.vert")];
    for shader in shaders {
        let loaded = shader::load_shader(shader, gl::VERTEX_SHADER);
        match loaded {
            Ok(s) => {
                println!("Shader OK");
                unsafe { s.delete() };
            }
            Err(e) => {
                println!("Had Error:\n{}", e);
            }
        }
    }
    let vert = shader::load_shader(include_str!("hello_cg.vert"), gl::VERTEX_SHADER)?;
    let frag = shader::load_shader(include_str!("hello_cg.frag"), gl::FRAGMENT_SHADER)?;
    let program = sh_program::load_program(vert, frag)?;
    unsafe {
        gl::UseProgram(program.into());
        gl::Viewport(0, 0, 960, 544);
        gl::ClearColor(0.0, 0.0, 0.5, 1.0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, VERTICES.as_ptr() as _);
        gl::EnableVertexAttribArray(0);
        loop {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            vglSwapBuffers(gl::FALSE);
        }
    }
    unsafe {
        program.delete();
        vert.delete();
        frag.delete();
    }
    Ok(())
}

fn main() {
    unsafe { setup_gl() };
    if let Err(e) = main_but_errors() {
        println!("Error: {}", e);
    }
    println!("---- RUN END ----");
}
