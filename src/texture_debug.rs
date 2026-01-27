use vita_gl_helpers::{
    attribute::{AttributeFormat, AttributeTable},
    attribute_table,
    buffer::{Buffer, GenDelBuffersExt},
    draw::{Elements, ElementsU16, Mode},
    program::{Program, link_program},
    shader::{Shader, load_shader},
    texture::Texture,
    uniform_table,
};

attribute_table!(TexDebugAttributeTable,
  uv => "aUv",
  pos => "aPos"
);

uniform_table!(TexDebugUniformTable,
  the_texture: Uniform1iv => "the_texture"
);

pub struct TexDebug {
    vert: Shader,
    frag: Shader,
    program: Program,
    attrs: TexDebugAttributeTable,
    unifs: TexDebugUniformTable,
    vbos: [Buffer; 2],
}

const FORMAT_F32X2: AttributeFormat = AttributeFormat {
    normalized: false,
    size: vita_gl_helpers::attribute::AttributeSize::TWO,
    type_: vita_gl_helpers::attribute::AttributeType::Float,
};

impl TexDebug {
    pub fn new() -> TexDebug {
        let vert = load_shader("
            void main(float2 aPos,float2 aUv,float4 out gl_Position : POSITION,float2 out vUv : TEXCOORD0) {
              gl_Position = float4(aPos,0.0,1.0);
              vUv = aUv;
            }
            ", gl::VERTEX_SHADER).expect("oops!");
        let frag = load_shader(
            "
            sampler2D uniform the_texture;
            float4 main(float2 in vUv : TEXCOORD0): COLOR {
                return tex2D(the_texture,vUv);
            }
            ",
            gl::FRAGMENT_SHADER,
        )
        .expect("oops!");
        let program = link_program(vert, frag).expect("oops!");
        let attrs = program.get_attribute_table().expect("oops!");
        let unifs = program.get_uniform_table().expect("oops!");
        let mut vbos = [Buffer::default(); 2];
        vbos.gen_buffers();
        vbos[0].data(
            gl::ARRAY_BUFFER,
            &[-1.0f32, 1., 1., 1., -1., -1., 1., -1.],
            gl::STATIC_DRAW,
        );
        vbos[1].data(
            gl::ARRAY_BUFFER,
            &[0.0f32, 0., 1., 0., 0., 1., 1., 1.],
            gl::STATIC_DRAW,
        );
        TexDebug {
            vert,
            frag,
            program,
            attrs,
            unifs,
            vbos,
        }
    }
    pub fn draw(&self, tex: Texture) {
        unsafe {
            self.program.use_me();
            gl::ActiveTexture(gl::TEXTURE0);
            tex.bind(gl::TEXTURE_2D);
            self.unifs.the_texture.set(0);
            self.attrs.enable_all();
            self.vbos[0].bind_to(self.attrs.pos, FORMAT_F32X2, 0, 0);
            self.vbos[1].bind_to(self.attrs.uv, FORMAT_F32X2, 0, 0);
            ElementsU16 {
                indices: &[0, 1, 3, 2],
            }
            .draw(Mode::Quads);
        }
    }
}

impl Drop for TexDebug {
    fn drop(&mut self) {
        unsafe {
            self.program.delete();
            self.frag.delete();
            self.vert.delete();
        }
    }
}
