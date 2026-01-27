#![feature(allocator_api)]

pub mod char_manager;
pub mod default_pal;
pub mod texture_debug;
pub mod vgl_allocator;

use vita_gl_helpers::{errors::eprintln_errors, initialise_default, swap_buffers};

use crate::{
    char_manager::CharMap,
    default_pal::{PAL_16, PAL_256},
    texture_debug::TexDebug,
};

pub const VERTICES: &'static [f32] = &[-0.7, 0.7, 0., 0.7, 0.7, 0., -0.7, -0.7, 0., 0.7, -0.7, 0.];
pub const UVS: &'static [f32] = &[0., 0., 1., 0., 0., 1., 1., 1.];

fn main_but_errors() -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    println!("---- RUN START ----");
    unsafe {
        gl::Enable(gl::TEXTURE_2D);
        gl::ActiveTexture(gl::TEXTURE0);
    }
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
    for r in 1..16 {
        for (i, c) in "Hello World!".chars().enumerate() {
            char_manager.put_char_16(c, r, 0, r - 1, i);
        }
    }
    let texdebug = TexDebug::new();
    unsafe {
        loop {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            char_manager.draw();
            // texdebug.draw(char_manager.textures[0]);
            eprintln_errors();
            swap_buffers();
        }
    }
}

fn main() {
    initialise_default();
    let Err(e) = main_but_errors();
    println!("Error: {}", e);
    println!("---- RUN END ----");
}
