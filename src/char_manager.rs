use std::cmp::Ordering;

use psf2_font::Psf2Font;
use vita_gl_helpers::{
    attribute::{AttributeFormat, AttributeTable},
    attribute_table,
    buffer::{Buffer, GenDelBuffersExt},
    draw::{Elements, ElementsU16},
    program::{Program, link_program},
    shader::{Shader, load_shader},
    texture::{GenDelTexturesExt, Texture},
    uniform_table,
};

use crate::font_rasterizer::{RasterizedFont, rasterize_font};

pub const QUAD_INDICES: &[u16] = &[0, 1, 3, 2];

pub const FORMAT_U8X4: AttributeFormat = AttributeFormat {
    normalized: false,
    size: vita_gl_helpers::attribute::AttributeSize::FOUR,
    type_: vita_gl_helpers::attribute::AttributeType::UnsignedByte,
};

attribute_table!(TtyAttributeTable,
   uvfg => "uvfg",
   bg => "bg"
);

uniform_table!(TtyUniformTable,
    char_dim: Uniform2fv => "charDim",
    // transform: UniformMatrix3x2fv => "transform",
    the_texture: Uniform1iv => "the_texture"
);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TermColor {
    Pal(u8),
    True(u32),
}

impl TermColor {
    pub const BLACK: TermColor = TermColor::Pal(0);
    pub const WHITE: TermColor = TermColor::Pal(15);
    pub fn select(self, pal: &[u32; 256]) -> u32 {
        match self {
            TermColor::Pal(i) => pal[i as usize],
            TermColor::True(c) => c & 0xFFFFFF,
        }
    }
}

pub struct CharMap {
    pub font: Psf2Font,
    pub useless_buffer: Vec<glam::Vec2>,
    /// Width of the screen
    pub screen_width: usize,
    /// Height of the screen
    pub screen_height: usize,
    ///Lower bytes of characters on the screen, fused with their 24 bit foreground colors, uvfg in the shader
    pub screen_lower: Vec<u32>,
    ///24 bit background colors, bg in the shader
    pub screen_bg: Vec<u32>,
    ///Upper bytes of characters on the screen, used to select textures
    pub screen_upper: Vec<u8>,
    ///Color to use when resizing/scrolling
    pub default_bg: TermColor,
    ///Vertex Buffer Objects
    pub vbos: [Buffer; 2],
    /// Vertex shader used to draw with
    pub vertex_shader: Shader,
    /// Fragment shader used to draw with
    pub fragment_shader: Shader,
    /// Program used to draw with
    pub program: Program,
    pub rasterized_font: RasterizedFont,
    pub pal_256: Box<[u32; 256]>,
    pub transform: [glam::Vec3; 2],
    // pub u_big_ass_uniform_location: i32,
    // pub u_other_big_ass_uniform_location: i32,
    pub uniforms: TtyUniformTable,
    pub attributes: TtyAttributeTable,
}

impl CharMap {
    fn compile_link_shaders(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tty_true = include_str!("ttytrue.vert");
        let screen_width = self.screen_width;
        let (_, tty_true) = tty_true
            .split_once('\n')
            .expect("okay who changed ttytrue.vert to not have a newline in it");
        let tty_true = format!("#define termWidth {screen_width}.0\n{tty_true}");
        if !self.vertex_shader.is_null() {
            unsafe {
                self.vertex_shader.delete();
                self.program.delete();
            }
        }
        self.vertex_shader = load_shader(&tty_true, gl::VERTEX_SHADER)?;
        self.program = link_program(self.vertex_shader, self.fragment_shader)?;
        self.uniforms = self.program.get_uniform_table()?;
        self.attributes = TtyAttributeTable::with_locations_from(&self.program)?;
        Ok(())
    }
    ///Gets the space character's lower and upper display values
    fn get_space(&self) -> (u32, u8) {
        let space_index = self
            .font
            .get_glyph_index(' ')
            .expect("The font didn't have space, WTF?");
        let space_lower_fill = (((space_index & 0xFF) << 24) as u32) | self.pal_256[0];
        let space_upper = (space_index >> 8) as u8; //Usually 0, but who knows, maybe someone's going to pass a really messed up PSF into this function.
        (space_lower_fill, space_upper)
    }
    pub fn new(
        font: Psf2Font,
        screen_width: usize,
        screen_height: usize,
        // pal_16: Box<[u32; 16]>,
        pal_256: Box<[u32; 256]>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rasterized_font = rasterize_font(&font);
        let mut this = CharMap {
            font,
            useless_buffer: vec![
                glam::Vec2::ZERO,
                glam::Vec2::X,
                glam::Vec2::Y,
                glam::Vec2::ONE,
            ],
            screen_width: 0,
            screen_height: 0,
            screen_lower: Vec::new(),
            screen_bg: Vec::new(),
            screen_upper: Vec::new(),
            default_bg: TermColor::Pal(0),
            vbos: [Default::default(); 2],
            vertex_shader: Shader::from(0),
            fragment_shader: load_shader(include_str!("tty.frag"), gl::FRAGMENT_SHADER)?,
            program: Program::from(0),
            rasterized_font,
            // pal_16,
            pal_256,
            transform: [glam::Vec3::ZERO; 2],
            uniforms: Default::default(),
            attributes: Default::default(),
        };
        this.vbos.gen_buffers();
        this.resize(screen_width, screen_height)?;
        this.compile_link_shaders()?;
        Ok(this)
    }
    pub fn resize(
        &mut self,
        term_width: usize,
        term_height: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (fill_lower, fill_upper) = self.get_space();
        let fill_bg = self.pal_256[0];
        match term_width.cmp(&self.screen_width) {
            Ordering::Less => {
                //We need to truncate each row, meaning we should make new vecs
                self.screen_lower = trunc_row(&self.screen_lower, self.screen_width, term_width);
                self.screen_upper = trunc_row(&self.screen_upper, self.screen_width, term_width);
                self.screen_bg = trunc_row(&self.screen_bg, self.screen_width, term_width);
            }
            Ordering::Equal => {
                //We don't have to do anything, lucky!
            }
            Ordering::Greater => {
                //We need to expand each row, meaning we should make new vecs
                self.screen_lower = expand_row(
                    &self.screen_lower,
                    self.screen_width,
                    term_width,
                    fill_lower,
                );
                self.screen_upper = expand_row(
                    &self.screen_upper,
                    self.screen_width,
                    term_width,
                    fill_upper,
                );
                self.screen_bg =
                    expand_row(&self.screen_bg, self.screen_width, term_width, fill_bg);
            }
        }
        self.screen_width = term_width;
        let new_n_items = term_width * term_height;
        self.screen_lower.resize(new_n_items, fill_lower);
        self.screen_upper.resize(new_n_items, fill_upper);
        self.screen_bg.resize(new_n_items, fill_bg);
        self.screen_height = term_height;
        Ok(())
    }
    pub fn draw(&self) {
        unsafe {
            gl::Enable(gl::TEXTURE_2D);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.rasterized_font.textures[0].bind(gl::TEXTURE_2D);
        let chrcount = (self.screen_width * self.screen_height) as i32;
        self.program.use_me();
        self.attributes.enable_all();
        self.vbos[0].bind_then(gl::ARRAY_BUFFER, |b| {
            b.data(&self.screen_lower, gl::DYNAMIC_DRAW);
            b.bind_to(self.attributes.uvfg, FORMAT_U8X4, 0, 0);
        });
        self.attributes.uvfg.divisor(1);
        self.vbos[1].bind_then(gl::ARRAY_BUFFER, |b| {
            b.data(&self.screen_bg, gl::DYNAMIC_DRAW);
            b.bind_to(self.attributes.bg, FORMAT_U8X4, 0, 0);
        });
        self.attributes.bg.divisor(1);
        self.uniforms
            .char_dim
            .set(bytemuck::cast(self.rasterized_font.char_dim));
        self.uniforms.the_texture.set(0);
        ElementsU16 {
            indices: QUAD_INDICES,
        }
        .draw_instanced(vita_gl_helpers::draw::Mode::Quads, chrcount);
        println!(
            "DRAW FG {:0>8x} BG {:0>8x}",
            self.screen_lower[0], self.screen_bg[0],
        );
    }
    fn lower_upper(&self, c: char) -> (u32, u8) {
        let i = self
            .font
            .get_glyph_index(c)
            .or_else(|| self.font.get_glyph_index('�'))
            .or_else(|| self.font.get_glyph_index('?'))
            .expect("No char or replacement or question mark found... Huh!?");
        (((i & 0xFF) << 24) as u32, (i >> 8) as u8)
    }
    fn select_default_bg(&self) -> u32 {
        self.default_bg.select(&self.pal_256)
    }
    fn put_lower_upper_bg(&mut self, row: usize, col: usize, lower: u32, upper: u8, bg: u32) {
        let loc = (row * self.screen_width) + col;
        self.screen_lower[loc] = lower;
        self.screen_upper[loc] = upper;
        self.screen_bg[loc] = bg;
    }
    // pub fn put_char_16(&mut self, c: char, fg: usize, bg: usize, row: usize, col: usize) {
    //     let (lower, upper) = self.lower_upper(c);
    //     let lower = lower | self.pal_16[fg];
    //     let bg = self.pal_16[bg];
    //     self.put_lower_upper_bg(row, col, lower, upper, bg);
    // }
    pub fn put_char_256(&mut self, c: char, fg: usize, bg: usize, row: usize, col: usize) {
        let (lower, upper) = self.lower_upper(c);
        let lower = lower | self.pal_256[fg];
        let bg = self.pal_256[bg];
        self.put_lower_upper_bg(row, col, lower, upper, bg);
    }
    pub fn put_char_true(&mut self, c: char, fg: u32, bg: u32, row: usize, col: usize) {
        let (lower, upper) = self.lower_upper(c);
        let lower = lower | fg;
        self.put_lower_upper_bg(row, col, lower, upper, bg);
    }
    pub fn put_char_tc(&mut self, c: char, fg: TermColor, bg: TermColor, row: usize, col: usize) {
        let (lower, upper) = self.lower_upper(c);
        let lower = lower | fg.select(&self.pal_256);
        let bg = bg.select(&self.pal_256);
        self.put_lower_upper_bg(row, col, lower, upper, bg);
    }
    pub fn scroll_up(&mut self, n_lines: usize) {
        let n_chars = self.screen_width * n_lines;
        self.screen_lower.copy_within(n_chars.., 0);
        self.screen_upper.copy_within(n_chars.., 0);
        self.screen_bg.copy_within(n_chars.., 0);
        let (space_lower, space_upper) = self.get_space();
        let space_bg = self.select_default_bg();
        let other_range = self.screen_lower.len() - n_chars;
        self.screen_lower[other_range..].fill(space_lower);
        self.screen_upper[other_range..].fill(space_upper);
        self.screen_bg[other_range..].fill(space_bg);
    }
    pub fn clear_screen(&mut self) {
        let (space_lower, space_upper) = self.get_space();
        let space_bg = self.select_default_bg();
        self.screen_lower.fill(space_lower);
        self.screen_upper.fill(space_upper);
        self.screen_bg.fill(space_bg);
    }
}

fn trunc_row<T: Copy>(old: &Vec<T>, oldsz: usize, newsz: usize) -> Vec<T> {
    let mut newvec = Vec::new();
    newvec.extend(
        old.chunks(oldsz)
            .flat_map(|chunk| &chunk[0..newsz])
            .map(|&x| x),
    );
    newvec
}

fn expand_row<T: Copy>(old: &Vec<T>, oldsz: usize, newsz: usize, fill: T) -> Vec<T> {
    let mut newvec = Vec::new();
    if oldsz == 0 {
        newvec.resize(newsz, fill);
        return newvec;
    }
    newvec.extend(old.chunks(oldsz).flat_map(|chunk| {
        chunk
            .into_iter()
            .map(|&x| x)
            .chain(std::iter::repeat(fill).take(newsz - oldsz))
    }));
    newvec
}
