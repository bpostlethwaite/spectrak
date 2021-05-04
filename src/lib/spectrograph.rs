use cgmath::{Matrix4, Vector2};
use std::borrow::Cow;

fn make_image_vec(dimx: u32, dimy: u32) -> Vec<f32> {
    let mut data = vec![];
    for _i in 0..dimx * dimy {
        data.push(0.3);
    }

    data
}

#[derive(Clone, Copy)]
struct Vertex {
    // The fields in Vertex are usually there
    // to be passed into the shader file.
    position: [f32; 2],
}

// This line implements the Vertex using a macro inside glium.
// Don't forget to include all of the fields as parameters otherwise
// glium won't pass those into the shader.
implement_vertex!(Vertex, position);

const VERTEX_SHADER: &'static str = r#"
    #version 140

    // Input parameter from the Vertex struct.
    in vec2 position;

    // Uniform parameters passed in from the frame.draw() call.
    uniform float offset;

    // Output texture coordinates that gets passed into the fragment shader.
    out vec2 v_tex_coords;

    void main() {
        // In order to return the texture coordinate for a specific
        // vertex we have to know what vertex is currently being passed in.
        // We do this through gl_VertexID which increments with every vertex passed in.
        // We can figure out the rectangle specific index from the vertex id by modding it
        // by 4. Example: if a vertex has id 16, then it is the first vertex of the fourth
        // rectangle being drawn. 16 % 4 == 0 which correctly returns the first index.

        if (gl_VertexID % 4 == 0) { // First vertex
            v_tex_coords = vec2(0.0, 1.0 + offset);
        } else if (gl_VertexID % 4 == 1) { // Second vertex
            v_tex_coords = vec2(1.0, 1.0 + offset);
        } else if (gl_VertexID % 4 == 2) { // Third vertex
            v_tex_coords = vec2(0.0, 0.0 + offset);
        } else { // Fourth vertex
            v_tex_coords = vec2(1.0, 0.0 + offset);
        }

        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    #version 140

    // Input texture coordinates passed from the vertex shader.
    in vec2 v_tex_coords;

    // Outputs the color for the specific fragment.
    out vec4 color;

    // Uniform parameter passed in from the frame.draw() call.
    uniform sampler2D data_tex;
    uniform sampler1D color_tex;

    float data_val;

    void main() {
        // Applies a texture to the rectangle.
        data_val = texture(data_tex, v_tex_coords).s;
        color = texture(color_tex, data_val);
    }
"#;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

pub struct Spectrograph {
    offset: f32,
    offset_idx: u32,
    width: u32,
    height: u32,
    tex_width: u32,
    tex_height: u32,
    data_texture: glium::texture::texture2d::Texture2d,
    color_texture: glium::texture::srgb_texture1d::SrgbTexture1d,
    vertex_position: egui::Rect,
    rect_vertices: glium::VertexBuffer<Vertex>,
    rect_indices: glium::IndexBuffer<u16>,
    rect_program: glium::Program,
}

impl Spectrograph {
    pub fn new(
        display: &glium::Display,
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    ) -> Spectrograph {
        let mut black_to_green = vec![
            1., 2., 3., 1., 65., 34., 1., 129., 65., 0., 192., 96., 0., 255., 127.,
        ];
        black_to_green.iter_mut().for_each(|i| *i /= 255.0);

        let mipmap = glium::texture::MipmapsOption::NoMipmap;
        let format = glium::texture::UncompressedFloatFormat::F32;
        let data_texture = glium::texture::texture2d::Texture2d::empty_with_format(
            display, format, mipmap, tex_width, tex_height,
        )
        .unwrap();

        let color_image = glium::texture::RawImage1d::from_raw_rgb(black_to_green);
        let color_texture =
            glium::texture::srgb_texture1d::SrgbTexture1d::new(display, color_image).unwrap();

        let image_vec = make_image_vec(tex_width, tex_height);

        let data_image = glium::texture::RawImage2d {
            data: Cow::from(&image_vec),
            width: tex_width,
            height: tex_height,
            format: glium::texture::ClientFormat::F32,
        };

        data_texture.write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width: tex_width,
                height: tex_height,
            },
            data_image,
        );

        let (rect_vertices, rect_indices) = {
            let ib_data: Vec<u16> = vec![0, 1, 2, 1, 3, 2];
            let vb = glium::VertexBuffer::empty_dynamic(display, 4).unwrap();
            let ib = glium::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &ib_data,
            )
            .unwrap();

            (vb, ib)
        };

        let rect_program =
            glium::Program::from_source(display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

        let vertex_position = egui::Rect {
            min: egui::Pos2 { x: -1.0, y: -1.0 },
            max: egui::Pos2 { x: 1.0, y: 1.0 },
        };

        Spectrograph {
            offset: 0.0,
            offset_idx: 0,
            width,
            height,
            data_texture,
            color_texture,
            rect_program,
            rect_vertices,
            rect_indices,
            vertex_position,
            tex_width,
            tex_height,
        }
    }

    pub fn update(&mut self, data: Vec<f32>) {
        self.offset = ((self.offset_idx + 1) as f32) / (self.height as f32);
        self.data_texture.write(
            glium::Rect {
                left: 0,
                bottom: self.offset_idx,
                width: self.tex_width,
                height: 1,
            },
            glium::texture::RawImage2d {
                data: Cow::from(data),
                width: self.tex_width,
                height: 1,
                format: glium::texture::ClientFormat::F32,
            },
        );

        self.offset_idx = (self.offset_idx + 1) % self.height;
    }

    pub fn set_vertex_position(&mut self, place_rect: egui::Rect, screen_rect: egui::Rect) {
        // Vertex positions start in bottom left hand corner (-1, -1)
        // We translate the relative percentage placement positions of place_rect and screen_rect
        // into Vertex space (-1 -> 1) by taking the ratio (0->1) and multiplying by 2 and adding
        // to the base coordinate (-1, -1)
        let left = -1.0 + place_rect.min.x / screen_rect.max.x * 2.0;
        let right = -1.0 + place_rect.max.x / screen_rect.max.x * 2.0;
        let top = -1.0 + (screen_rect.max.y - place_rect.min.y) / screen_rect.max.y * 2.0;
        let bottom = -1.0 + (screen_rect.max.y - place_rect.max.y) / screen_rect.max.y * 2.0;
        self.vertex_position = egui::Rect {
            min: egui::Pos2 { x: left, y: top },
            max: egui::Pos2 {
                x: right,
                y: bottom,
            },
        };
        // println!("{} {} {} {}", left, right, top, bottom);
        // println!("screen rect = {:?}", screen_rect);
        // println!("place rect = {:?}", place_rect);
        // println!("vertex position = {:?}", self.vertex_position);
    }

    pub fn draw(&mut self, target: &mut glium::Frame) {
        let uniforms = uniform! {
            data_tex: glium::uniforms::Sampler::new(&self.data_texture)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat),
            color_tex: glium::uniforms::Sampler::new(&self.color_texture)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            offset: self.offset,

        };

        {
            let left = self.vertex_position.min.x;
            let right = self.vertex_position.max.x;
            let bottom = self.vertex_position.max.y;
            let top = self.vertex_position.min.y;
            let vb_data = vec![
                Vertex {
                    position: [left, top],
                },
                Vertex {
                    position: [right, top],
                },
                Vertex {
                    position: [left, bottom],
                },
                Vertex {
                    position: [right, bottom],
                },
            ];
            self.rect_vertices.write(&vb_data);
        }

        {
            use glium::Surface as _;
            target
                .draw(
                    &self.rect_vertices,
                    &self.rect_indices,
                    &self.rect_program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }
    }
}
