use std::f64::consts::PI;
use std::ffi::CString;
use std::path::Path;
use std::{fs, ptr};
use std::time::SystemTime;

use gl::types::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use image::{DynamicImage, EncodableLayout, GenericImageView};
use image::imageops::FilterType;
use rand::distributions::Uniform;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use winit::dpi::Pixel;
use winit::event::{Event, WindowEvent};
use winit::event::VirtualKeyCode::B;
use winit::event_loop::ControlFlow;

use crate::opengl::{Buffer, Graphics, image_format, ReadWriteAccess, Shader, ShaderError, ShaderProgram, Texture, VertexArray};

mod opengl;

const VERTEX_SHADER_SOURCE: &str = "#version 460
in vec2 position;
in vec2 vertexTexCoord;

out vec2 texCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texCoord = vertexTexCoord;
}
";

const FRAGMENT_SHADER_SOURCE: &str = "#version 460
out vec4 FragColor;

in vec2 texCoord;

uniform sampler2D texture0;

void main() {
    FragColor = texture(texture0, texCoord);
}
";

type Pos = [f32; 2];
type TextureCoords = [f32; 2];

#[repr(C, packed)]
struct Vertex(Pos, TextureCoords);

#[rustfmt::skip]
const VERTICES: [Vertex; 4] = [
    Vertex([-1.0, -1.0], [0.0, 1.0]),
    Vertex([1.0, -1.0], [1.0, 1.0]),
    Vertex([1.0, 1.0], [1.0, 0.0]),
    Vertex([-1.0, 1.0], [0.0, 0.0]),
];

#[rustfmt::skip]
const INDICES: [i32; 6] = [
    0, 1, 2,
    2, 3, 0
];

#[repr(C, packed)]
struct OpenGLAgent {
    facing: f32,
    rotation_speed: f32,
    position: (f32, f32),

    min_speed: f32,
    max_speed: f32,
    rotation_bias: f32,
    p3: f32,

    sense_distance: u32,
    sense_angle: f32,
    sense_size: u32,
    deposition_amount: f32,

    color: (f32, f32, f32, f32),
}
// const TEXTURE_WIDTH: GLuint = 1920;
const TEXTURE_WIDTH: GLuint = 1080*2;
const TEXTURE_HEIGHT: GLuint = 1080*2;

unsafe fn run(width: u32, height: u32, scale: f32) -> Result<(), ShaderError> {
    let mut rng: Pcg64 = Seeder::from("12450").make_rng();
    let (graphics, event_loop) = Graphics::new(width, height, scale);

    let vertex_shader = Shader::new(VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER)?;
    let fragment_shader = Shader::new(FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER)?;
    let program = ShaderProgram::new(&[vertex_shader, fragment_shader])?;

    let vertex_array = VertexArray::new();
    vertex_array.bind();

    let vertex_buffer = Buffer::new(gl::ARRAY_BUFFER);
    vertex_buffer.set_data(&VERTICES, gl::STATIC_DRAW);

    let index_buffer = Buffer::new(gl::ELEMENT_ARRAY_BUFFER);
    index_buffer.set_data(&INDICES, gl::STATIC_DRAW);

    let pos_attrib = program.get_attrib_location("position")?;
    set_attribute!(vertex_array, pos_attrib, Vertex::0);
    let tex_coord = program.get_attrib_location("vertexTexCoord")?;
    set_attribute!(vertex_array, tex_coord, Vertex::1);

    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl::Enable(gl::BLEND);

    let mut major: GLint = 0;
    gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
    let mut minor: GLint = 0;
    gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);

    println!("GLVersion {major}.{minor}");

    let agents_shader = fs::read_to_string(Path::new("shaders/agents.comp")).unwrap();

    let agents_shader = Shader::new(&agents_shader, gl::COMPUTE_SHADER).unwrap();
    let agents_program = ShaderProgram::new(&[agents_shader]).unwrap();
    agents_program.apply();

    let mut data: Vec<[f32; 4]> = vec![[0.0; 4]; (TEXTURE_HEIGHT * TEXTURE_WIDTH) as usize];
    let trails_texture = Texture::new(image_format::RGBAFloat {});
    trails_texture.set_data(TEXTURE_WIDTH as i32, TEXTURE_HEIGHT as i32, &data);
    trails_texture.bind_image(0, ReadWriteAccess::ReadWrite);

    // data
    let mut agents: Vec<OpenGLAgent> = Vec::with_capacity(5);
    for i in 0..(0.02 * (TEXTURE_WIDTH as f32 * TEXTURE_HEIGHT as f32)) as u32 {
        agents.push(OpenGLAgent {
            // position: (rng.gen_range(0.0..TEXTURE_WIDTH as f32), rng.gen_range(0.0..TEXTURE_HEIGHT as f32)),
            position: (TEXTURE_WIDTH as f32 / 2.0 + rng.sample(Uniform::new(-10.0, 10.0)), TEXTURE_HEIGHT as f32 / 2.0  + rng.sample(Uniform::new(-10.0, 10.0))),
            facing: rng.sample(Uniform::new(-PI, PI)) as f32,
            rotation_speed: PI as f32 / 8.0,

            min_speed: 0.75,
            max_speed: 1.0,
            rotation_bias: 1.0 - ((i % 2) * 2) as f32,
            p3: 0.0,

            sense_distance: 9,
            sense_angle: PI as f32 / 8.0,
            sense_size: 1,
            deposition_amount: 5.0,

            color: (1.0, 0.0, 1.0, 1.0),
        });
    }

    let agents_buffer = Buffer::new(gl::SHADER_STORAGE_BUFFER);
    agents_buffer.set_data(&agents, gl::STATIC_DRAW);

    let mut trails_buffer = Buffer::new(gl::SHADER_STORAGE_BUFFER);
    let trails_starting_data = vec![1.0 as f32; (TEXTURE_WIDTH * TEXTURE_HEIGHT) as usize];
    trails_buffer.set_data(&trails_starting_data, gl::STATIC_DRAW);

    let mut new_trails_buffer = Buffer::new(gl::SHADER_STORAGE_BUFFER);
    new_trails_buffer.set_data(&trails_starting_data, gl::STATIC_DRAW);

    let pre_pattern = image::open(Path::new("assets/pre_pattern_three.png")).unwrap()
        .resize_exact(TEXTURE_WIDTH, TEXTURE_HEIGHT, FilterType::Gaussian)
        .into_rgb32f();
    let pre_pattern_data: Vec<_> = pre_pattern.into_iter().step_by(3).map(|v| *v).collect();

    let pre_pattern_buffer = Buffer::new(gl::SHADER_STORAGE_BUFFER);
    pre_pattern_buffer.set_data(&pre_pattern_data, gl::STATIC_DRAW);

    // update trails
    let trails_shader = fs::read_to_string(Path::new("shaders/update_trails.comp")).unwrap();

    let trails_shader = Shader::new(&trails_shader, gl::COMPUTE_SHADER).unwrap();
    let trails_program = ShaderProgram::new(&[trails_shader]).unwrap();

    let mut time = SystemTime::now();
    let mut frame = 0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let delta = time.elapsed().unwrap().as_secs_f64();

        if delta > 1.0 / 250.0 {
            frame += 1;
            if frame > 60 {
                println!("{}", 1.0 / delta);
                frame = 0;
            }

            time = SystemTime::now();

            // trails
            trails_program.apply();
            trails_program.set_int_uniform("trails", 0).unwrap();
            trails_buffer.bind_base(1);
            new_trails_buffer.bind_base(2);
            pre_pattern_buffer.bind_base(3);
            trails_texture.activate(gl::TEXTURE0);

            gl::DispatchCompute((TEXTURE_WIDTH as f32 / 32.0).ceil() as u32, (TEXTURE_HEIGHT as f32 / 32.0).ceil() as u32, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            std::mem::swap(&mut new_trails_buffer, &mut trails_buffer);

            // agents
            agents_program.apply();
            agents_program.set_int_uniform("trails", 0).unwrap();
            agents_buffer.bind_base(1);
            trails_buffer.bind_base(2);
            trails_texture.activate(gl::TEXTURE0);

            gl::DispatchCompute((agents.len() as f32 / 64.0).ceil() as u32, 1, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            // draw

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            program.apply();
            program.set_int_uniform("texture0", 0).unwrap();

            trails_texture.activate(gl::TEXTURE0);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            graphics.swap_buffers();

            match event {
                Event::LoopDestroyed => (),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        gl::Viewport(
                            0,
                            0,
                            physical_size.width as GLsizei,
                            physical_size.height as GLsizei,
                        );
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            }
        }
    });
}

fn main() {
    unsafe {
        run(TEXTURE_WIDTH, TEXTURE_HEIGHT, 1.25 / 2.0).unwrap();
    }
}
