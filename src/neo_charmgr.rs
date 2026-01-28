use psf2_font::Psf2Font;
use vita_gl_helpers::{
    attribute::{AttributeFormat, AttributeTable},
    attribute_table,
    buffer::{Buffer, GenDelBuffersExt},
    draw::{Elements, ElementsU16},
    program::{Program, link_program},
    shader::{Shader, load_shader},
    uniform_table,
};
use vt100::Parser;

use crate::font_rasterizer::{RasterizedFont, rasterize_font};

uniform_table!(FgUniformTable,
  transform : UniformMatrix3fv => "transform",
  char_dim : Uniform2fv => "char_dim",
  italic_shift : Uniform1fv => "italic_shift",
  the_texture: Uniform1iv => "the_texture"
);

attribute_table!(FgAttributeTable,
  uvxyst => "a_uvxyst",
  color => "a_color"
);

uniform_table!(BgUniformTable,
  transform : UniformMatrix3fv => "transform"
);

attribute_table!(BgAttributeTable,
  color => "a_color"
);

const COLOR_FORMAT: AttributeFormat = AttributeFormat {
    size: vita_gl_helpers::attribute::AttributeSize::FOUR,
    type_: vita_gl_helpers::attribute::AttributeType::UnsignedByte,
    normalized: true,
};

const TILEINFO_FORMAT: AttributeFormat = AttributeFormat {
    size: vita_gl_helpers::attribute::AttributeSize::FOUR,
    type_: vita_gl_helpers::attribute::AttributeType::UnsignedByte,
    normalized: false,
};

pub struct NeoCharRender {
    rasterized_font: RasterizedFont,
    big_buffer: Vec<u32>,
    big_buffer_vbo: Buffer,
    fg_texture_counts: Vec<usize>,
    /// Changing the width or height of this parser won't change the width or height this struct expects to see!
    pub parser: Parser,
    rows: usize,
    cols: usize,
    fg_program: Program,
    fg_vs: Shader,
    fg_fs: Shader,
    fg_unif_table: FgUniformTable,
    fg_attr_table: FgAttributeTable,
    bg_program: Program,
    bg_vs: Shader,
    bg_fs: Shader,
    bg_unif_table: BgUniformTable,
    bg_attr_table: BgAttributeTable,
}

impl NeoCharRender {
    /// Note we only support up to 256 rows and cols in each direction to keep the buffer smaller and of uniform type (a `u32` per cell).
    ///
    /// If you need more than that, use this crate as a reference and make your own terminal emulator!
    ///
    /// Also, if you need to change the size of the terminal, make a new `NeoCharRender`.
    pub fn new(
        font: &Psf2Font,
        max_row: u8,
        max_col: u8,
        scrollback_len: usize,
    ) -> Result<NeoCharRender, Box<dyn std::error::Error>> {
        let rasterized_font = rasterize_font(font);
        let parser = Parser::new((max_row as u16) + 1, (max_col as u16) + 1, scrollback_len);
        Self::new_with(rasterized_font, parser)
    }
    pub fn new_with(
        rasterized_font: RasterizedFont,
        parser: Parser,
    ) -> Result<NeoCharRender, Box<dyn std::error::Error>> {
        let (rows, cols) = parser.screen().size();
        if rows > 256 || cols > 256 {
            panic!("WHAT DID I TELL YOU ABOUT USING MORE THAN 256 ROWS OR COLUMNS?");
        }
        let n_tiles = (rows as usize) * (cols as usize);
        let big_buffer = vec![0u32; n_tiles * 3];
        let mut bbv = [Buffer::default()];
        bbv.gen_buffers();
        let big_buffer_vbo = bbv[0];
        let fg_vs = load_shader(include_str!("neo_tty_fg.vert"), gl::VERTEX_SHADER)?;
        let fg_fs = load_shader(include_str!("neo_tty_fg.frag"), gl::FRAGMENT_SHADER)?;
        let fg_program = link_program(fg_vs, fg_fs)?;
        let fg_unif_table = fg_program.get_uniform_table()?;
        let fg_attr_table = fg_program.get_attribute_table()?;
        let bg_vs = create_bg_vs(cols)?;
        let bg_fs = load_shader(include_str!("neo_tty_bg.frag"), gl::FRAGMENT_SHADER)?;
        let bg_program = link_program(bg_vs, bg_fs)?;
        let bg_unif_table = bg_program.get_uniform_table()?;
        let bg_attr_table = bg_program.get_attribute_table()?;
        let n_textures = rasterized_font.textures.len();
        Ok(NeoCharRender {
            rasterized_font,
            big_buffer,
            big_buffer_vbo,
            fg_texture_counts: vec![0; n_textures],
            parser,
            rows: rows as usize,
            cols: cols as usize,
            fg_program,
            fg_vs,
            fg_fs,
            fg_unif_table,
            fg_attr_table,
            bg_program,
            bg_vs,
            bg_fs,
            bg_unif_table,
            bg_attr_table,
        })
    }
    fn put_parser_data_into_buffers(&mut self, psf: &Psf2Font, pal: &[u32; 256]) {
        let n_tiles = self.rows * self.cols;
        let mut tileinfos: Vec<Vec<u32>> = vec![vec![]; self.rasterized_font.textures.len()];
        let mut tilefgs: Vec<Vec<u32>> = vec![vec![]; self.rasterized_font.textures.len()];
        let mut index = 0;
        let screen = self.parser.screen();
        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = screen
                    .cell(row as u16, col as u16)
                    .expect("WHY DON'T WE HAVE A CELL? DID YOU CHANGE THE SIZE OF THE PARSER?");
                let fg_color = map_color(cell.fgcolor(), pal, 0xFFFFFFFF);
                let bg_color = map_color(cell.bgcolor(), pal, 0xFF000000);
                let (fg_color, bg_color) = if cell.inverse() {
                    (bg_color, fg_color)
                } else {
                    (fg_color, bg_color)
                };
                self.big_buffer[index] = bg_color;
                let cell_char = cell.contents().chars().next().unwrap_or(' ');
                let cell_char_number = psf.get_glyph_index(cell_char).unwrap_or(0); //TODO: sub with replacement character, then space, then zero
                let texture_num = cell_char_number >> 8;
                //we can assume usize = u32
                let bit = |b, n| if b { 1usize << n } else { 0 };
                let tile_style = bit(cell.bold(), 0) | bit(cell.dim(), 1) | bit(cell.italic(), 2);
                let tile_info =
                    (cell_char_number & 0xFF) | (col << 8) | (row << 16) | (tile_style << 24);
                tileinfos[texture_num].push(tile_info as _);
                tilefgs[texture_num].push(fg_color);
                index += 1; //IMPORTANT
            }
        }
        for (tc, ti) in self.fg_texture_counts.iter_mut().zip(tileinfos.iter()) {
            *tc = ti.len();
        }
        let mut docpy = |start, vov: &[Vec<u32>]| {
            let mut index = start;
            for v in vov {
                self.big_buffer[index..(index + v.len())].copy_from_slice(v);
                index += v.len();
            }
        };
        docpy(n_tiles, &tilefgs);
        docpy(n_tiles * 2, &tileinfos);
    }
    //Unsure what the value of transform should be currently, so perhaps i'll have a tcp listener on another thread to receive values on?
    pub fn draw(&mut self, psf: &Psf2Font, pal: &[u32; 256], transform: [f32; 9]) {
        self.put_parser_data_into_buffers(psf, pal);
        let n_chars = self.rows * self.cols;
        self.big_buffer_vbo
            .data(gl::ARRAY_BUFFER, &self.big_buffer, gl::DYNAMIC_DRAW);
        self.bg_program.use_me();
        self.bg_unif_table.transform.set(transform, false); //Doesn't work, this function isn't loaded
        self.bg_attr_table.enable_all();
        let b = self.big_buffer_vbo.bind_then(gl::ARRAY_BUFFER, |b| b); //if it fits!
        b.bind_to(self.bg_attr_table.color, COLOR_FORMAT, 0, 0);
        self.bg_attr_table.color.divisor(1);
        quads(n_chars);
        self.fg_program.use_me();

        self.fg_attr_table.enable_all();
        self.fg_attr_table.color.divisor(1);
        self.fg_attr_table.uvxyst.divisor(1);

        self.fg_unif_table
            .char_dim
            .set(self.rasterized_font.char_dim.to_array());
        self.fg_unif_table.italic_shift.set(0.25); //hardcoded value for now because it's really not that important
        self.fg_unif_table.the_texture.set(0);

        unsafe {
            gl::Enable(gl::TEXTURE_2D);
            gl::ActiveTexture(gl::TEXTURE0);
            // gl::Enable(gl::BLEND);
            // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let mut index = 0;
        for (texindex, &n_to_draw) in self.fg_texture_counts.iter().enumerate() {
            self.rasterized_font.textures[texindex].bind(gl::TEXTURE_2D);
            b.bind_to(self.fg_attr_table.color, COLOR_FORMAT, 0, n_chars + index);
            b.bind_to(
                self.fg_attr_table.uvxyst,
                TILEINFO_FORMAT,
                0,
                (n_chars * 2) + index,
            );
            // quads(n_to_draw);
            index += n_to_draw;
        }
        unsafe {
            gl::Disable(gl::BLEND);
            gl::Disable(gl::TEXTURE_2D);
        }
    }
}

fn quads(count: usize) {
    ElementsU16 {
        indices: &[0, 1, 3, 2],
    }
    .draw_instanced(vita_gl_helpers::draw::Mode::Quads, count as _);
}

fn map_color(c: vt100::Color, p: &[u32; 256], d: u32) -> u32 {
    match c {
        vt100::Color::Default => d,
        vt100::Color::Idx(i) => p[i as usize] | 0xFF000000,
        vt100::Color::Rgb(r, g, b) => u32::from_ne_bytes([b, g, r, 0xFF]),
    }
}

fn create_bg_vs(width: u16) -> Result<Shader, Box<dyn std::error::Error>> {
    let bg_vs_source = include_str!("neo_tty_bg.vert");
    let (_, bg_vs_source) = bg_vs_source
        .split_once('\n')
        .expect("Why doesn't neo_tty_bg.vert have a single newline in it?");
    let bg_vs_source = format!("#define termWidth {width}.0\n{bg_vs_source}");
    Ok(load_shader(&bg_vs_source, gl::VERTEX_SHADER)?)
}

impl Drop for NeoCharRender {
    fn drop(&mut self) {
        unsafe {
            [self.big_buffer_vbo].del_buffers();
            self.fg_program.delete();
            self.fg_fs.delete();
            self.fg_vs.delete();
            self.bg_program.delete();
            self.bg_fs.delete();
            self.bg_vs.delete();
        }
    }
}
