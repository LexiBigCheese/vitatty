#![feature(allocator_api)]

pub mod char_manager;
pub mod default_pal;
pub mod font_rasterizer;
pub mod neo_charmgr;
pub mod texture_debug;
pub mod vgl_allocator;

use vita_gl_helpers::{errors::eprintln_errors, initialise_default, swap_buffers};

use crate::{
    char_manager::CharMap,
    default_pal::{PAL_16, PAL_256},
    neo_charmgr::NeoCharRender,
    texture_debug::TexDebug,
};

use std::io::{Read, Write};

pub const VERTICES: &'static [f32] = &[-0.7, 0.7, 0., 0.7, 0.7, 0., -0.7, -0.7, 0., 0.7, -0.7, 0.];
pub const UVS: &'static [f32] = &[0., 0., 1., 0., 0., 1., 1., 1.];

fn main_but_errors() -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    println!("---- RUN START ----");
    unsafe {
        gl::Enable(gl::TEXTURE_2D);
        gl::ActiveTexture(gl::TEXTURE0);
    }
    let terminus = psf2_font::load_terminus().expect("WAT");
    let mut neo_charmgr = NeoCharRender::new(&terminus, 25, 67, 0).expect("No NeoCharRender? sad");
    // neo_charmgr.parser.screen_mut().
    for i in 0..16 {
        writeln!(neo_charmgr.parser, "\x1B[48;5;{i}mHello World!\x1B[0m\r").unwrap();
    }
    neo_charmgr.parser.flush().unwrap();
    let texdebug = TexDebug::new();
    let transform_arc_mutex = std::sync::Arc::new(std::sync::Mutex::new([
        0.02f32, 0.0, -1.0, 0.0, -0.04, 1.0, 0.0, 0.0, 1.0,
    ]));
    let transform_arc_mutex_clone = transform_arc_mutex.clone();
    std::thread::spawn(move || {
        let transform_arc_mutex = transform_arc_mutex_clone;
        let listener = std::net::TcpListener::bind("0.0.0.0:9039").expect("NO BIND TO 9039? SCAM.");
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut the_string = String::new();
                    let Ok(_) = stream.read_to_string(&mut the_string) else {
                        continue;
                    };
                    println!("Received {the_string}");
                    let floats: Vec<f32> = the_string
                        .split(",")
                        .filter_map(|x| {
                            let y = x.trim().parse();
                            if let Err(e) = &y {
                                writeln!(stream, "BAD FLOAT {x}: {e:?}").unwrap();
                            };
                            y.ok()
                        })
                        .collect();
                    if floats.len() < 9 {
                        writeln!(stream, "NOT ENOUGH FLOATS").unwrap();
                    } else {
                        let mut acquired_lock =
                            transform_arc_mutex.lock().expect("CAN'T LOCK? SCAM");
                        acquired_lock.copy_from_slice(&floats[0..9]);
                        writeln!(stream, "OK").unwrap();
                    }
                    stream.flush().unwrap();
                    continue;
                }
                Err(e) => eprintln!("Aw fuck {e:?}"),
            }
        }
    });
    unsafe {
        loop {
            let the_transform = *transform_arc_mutex.lock().expect("WAT");
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            neo_charmgr.draw(&terminus, &PAL_256, the_transform);
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
