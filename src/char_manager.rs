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
    ///Vertex Buffer Objects
    pub vbos: [Buffer; 2],
    /// Vertex shader used to draw with
    pub vertex_shader: Shader,
    /// Fragment shader used to draw with
    pub fragment_shader: Shader,
    /// Program used to draw with
    pub program: Program,
    pub textures: Vec<Texture>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub pal_16: Box<[u32; 16]>,
    pub pal_256: Box<[u32; 256]>,
    pub char_dim: glam::Vec2,
    pub transform: [glam::Vec3; 2],
    // pub u_big_ass_uniform_location: i32,
    // pub u_other_big_ass_uniform_location: i32,
    pub uniforms: TtyUniformTable,
    pub attributes: TtyAttributeTable,
}

impl CharMap {
    fn gen_textures(&mut self) {
        if !self.textures.is_empty() {
            self.textures.delete_textures();
        }
        let (char_width, char_height) = self.font.dimensions();
        self.texture_width = (char_width * 16).next_power_of_two();
        self.texture_height = (char_height * 16).next_power_of_two();
        self.char_dim = {
            let c_dim = glam::vec2(char_width as f32, char_height as f32);
            let t_dim = glam::vec2(self.texture_width as f32, self.texture_height as f32);
            c_dim / t_dim
        };
        let n_textures_to_create = self.font.glyph_count().div_ceil(256);
        self.textures = vec![Texture::default(); n_textures_to_create];
        self.textures.gen_textures();
        //Keep this here to reuse the allocation. No need to clear as it will be overwritten.
        let mut tex_data = vec![0u8; self.texture_width as usize * self.texture_height as usize];
        let charcount = self.font.glyph_count();
        println!("We need to do {charcount} chars");
        for (tex_i, block) in ChunkIterator(self.font.glyph_count())
            .into_iter()
            .enumerate()
        {
            println!("ti {tex_i} block {block}");
            let tex_gl = self.textures[tex_i];

            for chr_index in 0..block {
                let glyph_index = (tex_i * 256) + chr_index;
                let glyph = self
                    .font
                    .get_glyph_by_index(glyph_index)
                    .expect("Somehow, got a char out of bounds");
                rasterize_char(
                    &mut tex_data,
                    self.texture_width as usize,
                    chr_index,
                    glyph,
                    char_width as usize,
                );
            }
            // dump_texture(&tex_data, self.texture_width as usize);
            tex_gl.bind_then(gl::TEXTURE_2D, |b| {
                b.image_2d(
                    0,
                    0x1909,
                    self.texture_width as i32,
                    self.texture_height as i32,
                    0x1909u32,
                    gl::UNSIGNED_BYTE,
                    tex_data.as_ptr() as _,
                );
                b.parameter_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                b.parameter_i(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            });
        }
    }
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
        let space_lower_fill = (((space_index & 0xFF) << 24) as u32) & self.pal_16[0];
        let space_upper = (space_index >> 8) as u8; //Usually 0, but who knows, maybe someone's going to pass a really messed up PSF into this function.
        (space_lower_fill, space_upper)
    }
    pub fn new(
        font: Psf2Font,
        screen_width: usize,
        screen_height: usize,
        pal_16: Box<[u32; 16]>,
        pal_256: Box<[u32; 256]>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
            vbos: [Default::default(); 2],
            vertex_shader: Shader::from(0),
            fragment_shader: load_shader(include_str!("tty.frag"), gl::FRAGMENT_SHADER)?,
            program: Program::from(0),
            textures: vec![],
            texture_width: 0,
            texture_height: 0,
            pal_16,
            pal_256,
            char_dim: glam::Vec2::ZERO,
            transform: [glam::Vec3::ZERO; 2],
            uniforms: Default::default(),
            attributes: Default::default(),
        };
        this.vbos.gen_buffers();
        this.resize(screen_width, screen_height)?;
        this.gen_textures();
        this.compile_link_shaders()?;
        Ok(this)
    }
    pub fn resize(
        &mut self,
        term_width: usize,
        term_height: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (fill_lower, fill_upper) = self.get_space();
        let fill_bg = self.pal_16[0];
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
        self.textures[0].bind(gl::TEXTURE_2D);
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
        self.uniforms.char_dim.set(bytemuck::cast(self.char_dim));
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
    fn put_lower_upper_bg(&mut self, row: usize, col: usize, lower: u32, upper: u8, bg: u32) {
        let loc = (row * self.screen_width) + col;
        self.screen_lower[loc] = lower;
        self.screen_upper[loc] = upper;
        self.screen_bg[loc] = bg;
    }
    pub fn put_char_16(&mut self, c: char, fg: usize, bg: usize, row: usize, col: usize) {
        let (lower, upper) = self.lower_upper(c);
        let lower = lower | self.pal_16[fg];
        let bg = self.pal_16[bg];
        self.put_lower_upper_bg(row, col, lower, upper, bg);
    }
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

struct ChunkIterator(usize);

impl Iterator for ChunkIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => None,
            n if n < 256 => {
                self.0 = 0;
                Some(n)
            }
            _ => {
                self.0 -= 256;
                Some(256)
            }
        }
    }
}

fn rasterize_char(
    target_array: &mut [u8],
    target_row_len: usize,
    target_uv: usize,
    data: &[u8],
    char_width: usize,
) {
    let mut ptr =
        ((target_uv >> 4) * target_row_len * data.len()) + ((target_uv & 0xF) * char_width);
    let stride = target_row_len - char_width;
    let mut char_width_iterator = 0;
    for &datum in data.into_iter() {
        let datum = datum.reverse_bits();
        for bit_n in 0..8 {
            let the_bool = ((datum >> bit_n) & 1) != 0;
            let the_byte = if the_bool { 0xFF } else { 0x00 };
            target_array[ptr] = the_byte;
            ptr += 1;
            char_width_iterator += 1;
            if char_width_iterator == char_width {
                ptr += stride;
                char_width_iterator = 0;
                break;
            }
        }
    }
}

#[allow(dead_code)]
fn dump_texture(tex: &[u8], width: usize) {
    for line in tex.chunks(width) {
        for &chr in line {
            print!("{}", if chr == 0xFF { "█" } else { " " });
        }
        println!("");
    }
}
