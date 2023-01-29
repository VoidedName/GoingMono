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
use vn_engine::color::{RGBA, WHITE};
use vn_engine::engine::{VNERunner, VNEngineState};
use vn_engine::render::{PixelPosition, VNERenderer};
use winit::dpi::Pixel;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::opengl::{Buffer, Graphics, Shader, ShaderError, ShaderProgram, Texture, VertexArray};

mod opengl;

struct Agent {
    sense_angle: f64,
    sense_distance: f64,
    sense_size: u32,

    speed: f64,
    facing: f64,
    rotation_speed: f64,
    position: (f64, f64),
    rotation_variance: f64,
}

impl Agent {
    fn sense(&self, world: &Grid<Cell>, angle: f64, bias: &mut Grid<f64>) -> f64 {
        let sense_pos = (
            self.position.0 + (self.facing + angle).cos() * self.sense_distance,
            self.position.1 - (self.facing + angle).sin() * self.sense_distance,
        );
        let sense_center = self.sense_size as f64 / 2.0;
        let mut sense_amount = 0.0;
        for w in 0..self.sense_size {
            for h in 0..self.sense_size {
                // let w = ((w as f64 - sense_center + sense_pos.0).round() + world.width as f64) % world.width as f64;
                // let h = ((h as f64 - sense_center + sense_pos.1).round() + world.height as f64) % world.height as f64;
                let w = (w as f64 - sense_center + sense_pos.0).round();
                let h = (h as f64 - sense_center + sense_pos.1).round();
                if w >= 0.0 && h >= 0.0 && w < world.width as f64 && h < world.height as f64 {
                    let patch = world.xy_index(w as usize, h as usize);
                    sense_amount +=
                        world.cells[patch].pheromone * bias.cells[patch] + bias.cells[patch];
                }
            }
        }
        sense_amount
    }

    fn turn_angle(&mut self, angle: f64) {
        self.facing = (self.facing + angle) % (2.0 * PI);
    }

    fn turn_right(&mut self) {
        self.turn_angle(-self.rotation_speed);
    }

    fn turn_left(&mut self) {
        self.turn_angle(self.rotation_speed);
    }

    fn update_facing(&mut self, rng: &mut Pcg64, world: &Grid<Cell>, bias: &mut Grid<f64>) {
        let left = self.sense(world, self.sense_angle, bias);
        let front = self.sense(world, 0.0, bias);
        let right = self.sense(world, -self.sense_angle, bias);

        if front > left && front > right {
            self.turn_angle(rng.sample(Uniform::new(
                -self.rotation_variance / 2.0,
                self.rotation_variance / 2.0,
            )))
        } else if front < right && front < right {
            if rng.gen_bool(0.5) {
                self.turn_left();
            } else {
                self.turn_right();
            };
        } else if left < right {
            self.turn_right()
        } else if right > left {
            self.turn_left();
        }
    }

    fn try_move(&mut self, rng: &mut Pcg64, world: &mut Grid<Cell>) {
        // let x = (self.position.0 + self.facing.cos() * self.speed + world.width as f64) % world.width as f64;
        // let y = (self.position.1 - self.facing.sin() * self.speed + world.height as f64) % world.height as f64;
        let x = self.position.0 + self.facing.cos() * self.speed;
        let y = self.position.1 - self.facing.sin() * self.speed;

        if x >= 0.0 && y >= 0.0 && x < world.width as f64 && y < world.height as f64 {
            self.position = (x, y);
            let idx = world.xy_index(x as usize, y as usize);
            let cell = &mut world.cells[idx];
            cell.pheromone += 5.0;
        } else {
            self.facing = rng.sample(Uniform::new(0.0, 2.0 * PI))
        }
    }

    fn update(&mut self, rng: &mut Pcg64, world: &mut Grid<Cell>, bias: &mut Grid<f64>) {
        self.update_facing(rng, world, bias);
        self.try_move(rng, world);
    }
}

#[derive(Copy, Clone)]
struct Cell {
    pheromone: f64,
}

impl Cell {
    fn new() -> Cell {
        Cell { pheromone: 0.0 }
    }
}

struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T> Grid<T> {
    pub fn xy_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

struct Simulation {
    world: Grid<Cell>,
    agents: Vec<Agent>,
    rng: Pcg64,
    bias: Grid<f64>,
}

impl VNERunner for Simulation {
    fn setup(&mut self, _: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        renderer.set_title("Slime, Slime everywhere!");
        renderer.fill_rectangle(
            PixelPosition { x: 0, y: 0 },
            PixelPosition {
                x: PIXEL_WIDTH,
                y: PIXEL_HEIGHT,
            },
            WHITE,
        );
    }

    fn tick(&mut self, _: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        let mut new_cells = Vec::with_capacity(self.world.width * self.world.height);
        let mut max: f64 = 0.0;
        let mut total: f64 = 0.0;
        for y in 0..(self.world.height as i32) {
            for x in 0..(self.world.width as i32) {
                let mut new_value = 0.0;
                for i in -1..=1 {
                    for j in -1..=1 {
                        // let nx = (x + i + self.world.width as i32) % self.world.width as i32;
                        // let ny = (y + j + self.world.height as i32) % self.world.height as i32;
                        let nx = x + i;
                        let ny = y + j;
                        if nx >= 0
                            && ny >= 0
                            && nx < self.world.width as i32
                            && ny < self.world.height as i32
                        {
                            let idx = self.world.xy_index(nx as usize, ny as usize);
                            new_value += self.world.cells[idx].pheromone / 9.0;
                        }
                    }
                }
                new_value = (new_value * (1.0 - PHEROMONE_EVAPORATION)).max(0.0);
                max = max.max(new_value);
                total += new_value;
                let new_cell = Cell {
                    pheromone: new_value,
                };
                new_cells.push(new_cell);
            }
        }
        self.world.cells = new_cells;

        for x in 0..(self.world.width) {
            for y in 0..(self.world.height) {
                let idx = self.world.xy_index(x, y);
                let intensity = (self.world.cells[idx].pheromone / max) as f32;
                renderer.fill_rectangle(
                    PixelPosition {
                        x: x as u32 * (PIXEL_WIDTH / self.world.width as u32),
                        y: y as u32 * (PIXEL_HEIGHT / self.world.height as u32),
                    },
                    PixelPosition {
                        x: (1 + x as u32) * (PIXEL_WIDTH / self.world.width as u32) - 1,
                        y: (1 + y as u32) * (PIXEL_HEIGHT / self.world.height as u32) - 1,
                    },
                    RGBA {
                        r: (intensity + self.bias.cells[idx] as f32).min(1.0),
                        g: intensity.max(0.0),
                        b: (intensity - self.bias.cells[idx] as f32).max(0.0),
                        a: 1.0,
                    },
                );
            }
        }

        // for agent in self.agents.iter() {
        //     let (x, y) = agent.position;
        //     renderer.draw_pixel(
        //         PixelPosition {
        //             x: (x * (PIXEL_WIDTH / self.world.width as u32) as f64) as u32,
        //             y: (y * (PIXEL_HEIGHT / self.world.height as u32) as f64) as u32,
        //         },
        //         RED,
        //     )
        // }
        for agent in self.agents.iter_mut() {
            agent.update(&mut self.rng, &mut self.world, &mut self.bias)
        }
    }
}

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

    // let width_tiles = PIXEL_WIDTH;
    // let height_tiles = PIXEL_HEIGHT;
    // let mut rng: Pcg64 = Seeder::from("12450").make_rng();
    //
    // let mut runner = Simulation {
    //     world: grid![Cell::new(), width_tiles as usize, height_tiles as usize],
    //     bias: grid![0.01, width_tiles as usize, height_tiles as usize],
    //     agents: vec![],
    //     rng: Seeder::from(rng.gen::<u128>()).make_rng(),
    // };
    //
    // for x in 0..runner.bias.width {
    //     for y in 0..runner.bias.height {
    //         if x < 60 && x > 40 && y > 90 && y < 110 {
    //             let idx = runner.bias.xy_index(x, y);
    //             runner.bias.cells[idx] = ((x as i32 - 50) as f64 / 10.0).cos() * ((y as i32 - 100) as f64 / 10.0).cos() * 0.1 + 1.1;
    //         }
    //
    //         if x < 160 && x > 140 && y > 90 && y < 110 {
    //             let idx = runner.bias.xy_index(x, y);
    //             runner.bias.cells[idx] = ((x as i32 - 150) as f64 / 10.0).cos() * ((y as i32 - 100) as f64 / 10.0).cos() * 0.1 + 1.1;
    //         }
    //     }
    //     // let x = rng.gen_range(100 .. width_tiles as usize - 100);
    //     // let y = rng.gen_range(50 .. height_tiles as usize - 50);
    //     // let bias_width = rng.sample(Uniform::new(35, 50));
    //     // let bias_center = bias_width as f64 / 2.0;
    //     // for i in 0..bias_width {
    //     //     for j in 0..bias_width {
    //     //         let nx = x+i;
    //     //         let ny = y+j;
    //     //         let strength_x = 1.0 - ((i as f64 - bias_center) / bias_center).abs();
    //     //         let strength_y = 1.0 - ((j as f64 - bias_center) / bias_center).abs();
    //     //         let idx = runner.bias.xy_index(nx, ny);
    //     //         runner.bias.cells[idx] = strength_x * strength_y;
    //     //     }
    //     // }
    // }
    //
    // // for x in 0..runner.bias.width {
    // //     for y in 0..50 {
    // //         let idx = runner.bias.xy_index(x, y);
    // //         let strength = y as f64 / 50.0 * 0.01;
    // //         runner.bias.cells[idx] = strength;
    // //
    // //         let idx = runner.bias.xy_index(x, runner.bias.height-y-1);
    // //         runner.bias.cells[idx] = strength;
    // //     }
    // // }
    // //
    // // for y in 0..runner.bias.height {
    // //     for x in 0..50 {
    // //         let idx = runner.bias.xy_index(x, y);
    // //         let strength = x as f64 / 50.0 * 0.01;
    // //         runner.bias.cells[idx] = strength;
    // //
    // //         let idx = runner.bias.xy_index(runner.bias.width-x-1, y);
    // //         runner.bias.cells[idx] = strength;
    // //     }
    // // }
    //
    // for _ in 0..(width_tiles as f64 * height_tiles as f64 * 0.02) as usize {
    //     runner.agents.push(Agent {
    //         sense_size: 1,
    //         sense_distance: 9.0,
    //         sense_angle: PI / 4.0,
    //         speed: 1.0,
    //         facing: rng.sample(Uniform::new(0.0, 2.0 * PI)),
    //
    //         position: (rng.gen_range(0..width_tiles) as f64, rng.gen_range(0..height_tiles) as f64),
    //         // position: (width_tiles as f64 / 2.0 + rng.sample(Uniform::new(-10.0, 10.0)), height_tiles as f64 / 2.0 + rng.sample(Uniform::new(-10.0, 10.0))),
    //
    //         rotation_variance: PI / 32.0,
    //         rotation_speed: PI / 8.0,
    //     })
    // }
    //
    // let mut engine = vn_engine::engine::VNEngine::new_sprite_based(PIXEL_WIDTH, PIXEL_HEIGHT, 2, false);
    // engine.run(&mut runner)
}
