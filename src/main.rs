pub mod char_manager;
pub mod default_pal;
pub mod sh_program;
pub mod shader;
pub mod tty_render;

use std::ffi::CString;

use crate::{
    char_manager::CharMap,
    default_pal::{PAL_16, PAL_256},
};

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

pub const VERTICES: &'static [f32] = &[-0.7, 0.7, 0., 0.7, 0.7, 0., -0.7, -0.7, 0., 0.7, -0.7, 0.];
pub const UVS: &'static [f32] = &[0., 0., 1., 0., 0., 1., 1., 1.];

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

fn main_but_errors() -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    println!("---- RUN START ----");
    let shaders = [include_str!("ttytrue.vert")];
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
    let mut texture = 0u32;
    unsafe {
        gl::Enable(gl::TEXTURE_2D);
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        let mut image_data = vec![0xFFFFu16; 128 * 128];
        for x in 0..128 {
            for y in 0..128 {
                let rg = ((y << 9) | (x << 1)) as u16;
                image_data[(y * 128) + x] = rg;
            }
        }
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RG8 as i32, //GL_RG
            128,
            128,
            0,
            gl::RG,
            gl::UNSIGNED_BYTE,
            image_data.as_ptr() as _,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        // glBegin(gl::QUADS);
        // glTexCoord2i(0, 1);
        // glVertex3f(0., 0., 0.);
        // glTexCoord2i(1, 1);
        // glVertex3f(960., 0., 0.);
        // glTexCoord2i(1, 0);
        // glVertex3f(960., 544., 0.);
        // glTexCoord2i(0, 0);
        // glVertex3f(0., 544., 0.);
        // glEnd();
    }
    let indices = vec![0u16, 1, 2, 1, 3, 2];
    let mut char_manager = Box::new(
        CharMap::new(
            psf2_font::load_terminus().expect("Aw fuck"),
            68,
            26,
            Box::new(PAL_16),
            Box::new(PAL_256),
        )
        .expect("aw fuck, i can't make charmap :("),
    );
    char_manager.transform = [glam::vec3(0.1, 0., -0.5), glam::vec3(0., 0.1, -0.5)];
    for r in 0..26 {
        for (i, c) in "Hello World!".chars().enumerate() {
            char_manager.put_char_16(c, 12, 9, r, i);
        }
    }
    unsafe {
        loop {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            // gl::ActiveTexture(gl::TEXTURE0);
            // gl::BindTexture(gl::TEXTURE_2D, texture);
            // gl::Uniform1i(the_texture_location, 0);
            // gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, indices.as_ptr() as _);
            char_manager.draw();
            vglSwapBuffers(gl::FALSE);
        }
    }
}

fn main() {
    unsafe { setup_gl() };
    let Err(e) = main_but_errors();
    println!("Error: {}", e);
    println!("---- RUN END ----");
}
