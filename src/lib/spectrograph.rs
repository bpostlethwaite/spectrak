use cgmath::{Matrix4, Vector2};
use std::borrow::Cow;

fn make_image_vec(dimx: u32, dimy: u32) -> Vec<f32> {
    let mut data = vec![];
    for _i in 0..dimx {
        for _j in 0..dimy {
            data.push(0.5);
        }
    }

    data
}

fn make_row(dimy: u32, val: f32) -> Vec<f32> {
    let mut data = vec![];
    for _i in 0..dimy {
        data.push(val);
    }
    data
}

fn make_gradient_row(xn: u32, offset: f32) -> Vec<f32> {
    make_row(xn, offset)
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
    uniform mat4 projection;
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
            v_tex_coords = vec2(0.0, 1.0 - offset);
        } else if (gl_VertexID % 4 == 1) { // Second vertex
            v_tex_coords = vec2(1.0, 1.0 - offset);
        } else if (gl_VertexID % 4 == 2) { // Third vertex
            v_tex_coords = vec2(0.0, 0.0 - offset);
        } else { // Fourth vertex
            v_tex_coords = vec2(1.0, 0.0 - offset);
        }

        gl_Position = projection * vec4(position, 0.0, 1.0);
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
    offset_idx: u32,
    width: u32,
    height: u32,
    data_texture: glium::texture::texture2d::Texture2d,
    color_texture: glium::texture::srgb_texture1d::SrgbTexture1d,
    rect_position: Vector2<f32>,
    rect_size: Vector2<f32>,
    rect_vertices: glium::VertexBuffer<Vertex>,
    rect_indices: glium::IndexBuffer<u16>,
    rect_program: glium::Program,
    perspective: [[f32; 4]; 4],
}

impl Spectrograph {
    pub fn new(display: &glium::Display, width: u32, height: u32) -> Spectrograph {
        let mut black_to_green = vec![
            1., 2., 3., 1., 65., 34., 1., 129., 65., 0., 192., 96., 0., 255., 127.,
        ];
        black_to_green.iter_mut().for_each(|i| *i /= 255.0);

        let mipmap = glium::texture::MipmapsOption::NoMipmap;
        let format = glium::texture::UncompressedFloatFormat::F32;
        let data_texture = glium::texture::texture2d::Texture2d::empty_with_format(
            display, format, mipmap, width, height,
        )
        .unwrap();

        let color_image = glium::texture::RawImage1d::from_raw_rgb(black_to_green);
        let color_texture =
            glium::texture::srgb_texture1d::SrgbTexture1d::new(display, color_image).unwrap();

        let image_vec = make_image_vec(width, height);

        let data_image = glium::texture::RawImage2d {
            data: Cow::from(&image_vec),
            width,
            height,
            format: glium::texture::ClientFormat::F32,
        };

        data_texture.write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width,
                height,
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

        let perspective = {
            let matrix: Matrix4<f32> = cgmath::ortho(
                0.0,
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
                0.0,
                -1.0,
                1.0,
            );
            Into::<[[f32; 4]; 4]>::into(matrix)
        };

        let rect_size = Vector2 {
            x: width as f32,
            y: height as f32,
        };

        let rect_position = Vector2 {
            x: (SCREEN_WIDTH / 2) as f32,
            y: (SCREEN_HEIGHT / 2) as f32,
        };

        Spectrograph {
            offset_idx: 0,
            width: 300,
            height: 600,
            data_texture,
            color_texture,
            rect_position,
            rect_size,
            rect_program,
            rect_vertices,
            rect_indices,
            perspective,
        }
    }

    pub fn draw(&mut self, target: &mut glium::Frame) {
        let offset = ((self.offset_idx + 1) as f32) / (self.height as f32);
        let row = make_gradient_row(self.width, offset);

        self.data_texture.write(
            glium::Rect {
                left: 0,
                bottom: self.height - (self.offset_idx + 1),
                width: self.width,
                height: 1,
            },
            glium::texture::RawImage2d {
                data: Cow::from(&row),
                width: self.width,
                height: 1,
                format: glium::texture::ClientFormat::F32,
            },
        );

        let uniforms = uniform! {
            data_tex: glium::uniforms::Sampler::new(&self.data_texture)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat),
            color_tex: glium::uniforms::Sampler::new(&self.color_texture)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),

            projection: self.perspective,
            offset: offset,
        };

        {
            let left = self.rect_position.x - self.rect_size.x / 2.0;
            let right = self.rect_position.x + self.rect_size.x / 2.0;
            let bottom = self.rect_position.y + self.rect_size.y / 2.0;
            let top = self.rect_position.y - self.rect_size.y / 2.0;
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

        self.offset_idx = (self.offset_idx + 1) % self.height;

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
