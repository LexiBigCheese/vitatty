use psf2_font::Psf2Font;
use vita_gl_helpers::texture::{GenDelTexturesExt, Texture};

pub struct RasterizedFont {
    pub textures: Vec<Texture>,
    pub texture_width: usize,
    pub texture_height: usize,
    pub char_dim: glam::Vec2,
}

impl Drop for RasterizedFont {
    fn drop(&mut self) {
        self.textures.delete_textures();
    }
}

pub fn rasterize_font(font: &Psf2Font) -> RasterizedFont {
    let (char_width, char_height) = font.dimensions();
    let texture_width = (char_width * 16).next_power_of_two() as usize;
    let texture_height = (char_height * 16).next_power_of_two() as usize;
    let char_dim = {
        let c_dim = glam::vec2(char_width as f32, char_height as f32);
        let t_dim = glam::vec2(texture_width as f32, texture_height as f32);
        c_dim / t_dim
    };
    let charcount = font.glyph_count();
    let n_textures_to_create = charcount.div_ceil(256);
    let mut textures = vec![Texture::default(); n_textures_to_create];
    textures.gen_textures();
    //Keep this here to reuse the allocation. No need to clear as it will be overwritten.
    let mut tex_data = vec![0u8; texture_width * texture_height];
    println!("We need to do {charcount} chars");
    for (tex_i, block) in ChunkIterator(charcount).into_iter().enumerate() {
        println!("ti {tex_i} block {block}");
        let tex_gl = textures[tex_i];

        for chr_index in 0..block {
            let glyph_index = (tex_i * 256) + chr_index;
            let glyph = font
                .get_glyph_by_index(glyph_index)
                .expect("Somehow, got a char out of bounds");
            rasterize_char(
                &mut tex_data,
                texture_width as usize,
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
                texture_width as i32,
                texture_height as i32,
                0x1909u32,
                gl::UNSIGNED_BYTE,
                tex_data.as_ptr() as _,
            );
            b.parameter_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            b.parameter_i(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        });
    }
    RasterizedFont {
        textures,
        texture_width,
        texture_height,
        char_dim,
    }
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
