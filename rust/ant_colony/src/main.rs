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
use winit::event_loop::ControlFlow;

use crate::opengl::{Buffer, Graphics, Shader, ShaderError, ShaderProgram, Texture, VertexArray};

mod opengl;

// const PIXEL_WIDTH: u32 = 200;
// const PIXEL_HEIGHT: u32 = 200;
const PIXEL_WIDTH: u32 = 1920;
const PIXEL_HEIGHT: u32 = 1080;
const PHEROMONE_EVAPORATION: f64 = 0.1;

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

    speed: f32,
    p1: f32,
    p2: f32,
    p3: f32,

    sense_distance: u32,
    sense_angle: f32,
    sense_size: u32,
    deposition_amount: f32,

    color: (f32, f32, f32, f32),
}
// const TEXTURE_WIDTH: GLuint = 1920;
const TEXTURE_WIDTH: GLuint = 1080;
const TEXTURE_HEIGHT: GLuint = 1080;

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

    let mut trails_image: GLuint = 0;

    gl::GenTextures(1, &mut trails_image);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, trails_image);
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as GLint,
    );
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as GLint,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    let mut data: Vec<f32> = vec![0.0; (TEXTURE_HEIGHT * TEXTURE_WIDTH * 4) as usize];
    let mut buffer = data;

    // let mut data = image::DynamicImage::new_rgba32f(TEXTURE_WIDTH, TEXTURE_HEIGHT);
    // let buffer = data.as_mut_rgba32f().unwrap();
    // buffer.fill(1.0);

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA32F as GLint,
        TEXTURE_WIDTH as GLsizei,
        TEXTURE_HEIGHT as GLsizei,
        0,
        gl::RGBA,
        gl::FLOAT,
        buffer.as_mut_ptr().cast(),
    );
    gl::BindImageTexture(0, trails_image, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA32F);

    // data
    let mut agents: Vec<OpenGLAgent> = Vec::with_capacity(5);
    for _ in 0..(0.05 * (TEXTURE_WIDTH as f32 * TEXTURE_HEIGHT as f32)) as u32 {
        agents.push(OpenGLAgent {
            position: (rng.gen_range(0.0..TEXTURE_WIDTH as f32), rng.gen_range(0.0..TEXTURE_HEIGHT as f32)),
            // position: (TEXTURE_WIDTH as f32 / 2.0 + rng.sample(Uniform::new(-1.0, 1.0)), TEXTURE_HEIGHT as f32 / 2.0  + rng.sample(Uniform::new(-1.0, 1.0))),
            facing: rng.sample(Uniform::new(-PI, PI)) as f32,
            rotation_speed: PI as f32 / 8.0,

            speed: 1.0,
            p1: 0.0,
            p2: 0.0,
            p3: 0.0,

            sense_distance: 9,
            sense_angle: PI as f32 / 8.0,
            sense_size: 1,
            deposition_amount: 5.0,

            color: (1.0, 0.0, 1.0, 0.0),
        });
    }

    let mut agents_buffer: GLuint = 0;
    gl::GenBuffers(1, &mut agents_buffer);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, agents_buffer);
    gl::BufferData(gl::SHADER_STORAGE_BUFFER, std::mem::size_of::<OpenGLAgent>() as GLsizeiptr * agents.len() as GLsizeiptr, agents.as_ptr().cast(), gl::STATIC_DRAW);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, agents_buffer);


    let trails_starting_data = vec![1.0 as f32; (TEXTURE_WIDTH * TEXTURE_HEIGHT) as usize];
    let mut trails_buffer: GLuint = 0;
    gl::GenBuffers(1, &mut trails_buffer);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, trails_buffer);
    gl::BufferData(gl::SHADER_STORAGE_BUFFER, std::mem::size_of::<f32>() as GLsizeiptr * (TEXTURE_WIDTH * TEXTURE_HEIGHT) as isize, trails_starting_data.as_ptr().cast(), gl::STATIC_DRAW);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, trails_buffer);

    let mut new_trails_buffer: GLuint = 0;
    gl::GenBuffers(1, &mut new_trails_buffer);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, new_trails_buffer);
    gl::BufferData(gl::SHADER_STORAGE_BUFFER, std::mem::size_of::<f32>() as GLsizeiptr * (TEXTURE_WIDTH * TEXTURE_HEIGHT) as isize, trails_starting_data.as_ptr().cast(), gl::STATIC_DRAW);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, new_trails_buffer);

    let pre_pattern = image::open(Path::new("assets/pre_pattern_haphazard.png")).unwrap()
        .resize_exact(TEXTURE_WIDTH, TEXTURE_HEIGHT, FilterType::Gaussian)
        .into_rgb32f();
    let pre_pattern_data: Vec<_> = pre_pattern.into_iter().step_by(3).map(|v| *v).collect();

    let mut pre_pattern_buffer: GLuint = 0;
    gl::GenBuffers(1, &mut pre_pattern_buffer);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, pre_pattern_buffer);
    gl::BufferData(gl::SHADER_STORAGE_BUFFER, std::mem::size_of::<f32>() as GLsizeiptr * (TEXTURE_WIDTH * TEXTURE_HEIGHT) as isize, pre_pattern_data.as_ptr().cast(), gl::STATIC_DRAW);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, pre_pattern_buffer);

    // update trails
    let trails_shader = fs::read_to_string(Path::new("shaders/update_trails.comp")).unwrap();

    let trails_shader = Shader::new(&trails_shader, gl::COMPUTE_SHADER).unwrap();
    let trails_program = ShaderProgram::new(&[trails_shader]).unwrap();
    trails_program.apply();
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, trails_buffer);

    let mut time = SystemTime::now();
    let mut frame = 0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let delta = time.elapsed().unwrap().as_secs_f64();

        if delta > 1.0 / 500.0 {
            frame += 1;
            if frame > 60 {
                println!("{}", 1.0 / delta);
                frame = 0;
            }

            time = SystemTime::now();

            // trails
            trails_program.apply();
            trails_program.set_int_uniform("trails", 0).unwrap();
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, trails_buffer);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, new_trails_buffer);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, pre_pattern_buffer);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, trails_image);

            gl::DispatchCompute((TEXTURE_WIDTH as f32 / 32.0).ceil() as u32, (TEXTURE_HEIGHT as f32 / 32.0).ceil() as u32, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            std::mem::swap(&mut new_trails_buffer, &mut trails_buffer);

            // agents
            agents_program.apply();
            agents_program.set_int_uniform("trails", 0).unwrap();
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, agents_buffer);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, trails_buffer);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, trails_image);

            gl::DispatchCompute((agents.len() as f32 / 64.0).ceil() as u32, 1, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

            // draw

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            program.apply();
            program.set_int_uniform("texture0", 0).unwrap();

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, trails_image);

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
        run(TEXTURE_WIDTH, TEXTURE_HEIGHT, 1.25).unwrap();
    }
}
