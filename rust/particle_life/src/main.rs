use crate::opengl::quad::Quad;
use crate::opengl::{Buffer, Graphics, Shader};
use crate::particle_life::{Force, Particle, ParticleForces, World};
use crate::utils::{Colour, Vec2};
use gl::types::GLuint;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::ops::Add;
use std::time::SystemTime;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod opengl;
mod particle_life;
mod utils;

const FRAGMENT_SHADER_SOURCE: &str = "#version 460
#ifdef GL_ES
precision mediump float;
#endif

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;
uniform float u_r;

in vec2 pos;
in vec4 c;

out vec4 FragColor;
void main() {
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    st.x *= u_resolution.x/u_resolution.y;

    vec2 circle_pow = pos.xy/u_resolution.xy;
    circle_pow.x *= u_resolution.x/u_resolution.y;

    float r = u_r / u_resolution.y;

    float a = distance(st, circle_pow);
    if (a > r) { discard; }
    FragColor = vec4(c);
}
";

const VERTEX_SHADER_SOURCE: &str = "#version 460
#ifdef GL_ES
precision mediump float;
#endif

in vec2 position;
in vec2 offset;
in vec4 color;

uniform vec2 u_resolution;
uniform float u_r;

out vec2 pos;
out vec4 c;
void main() {
    vec2 p = offset.xy / u_resolution.xy;
    p *= u_resolution.x/u_resolution.y;
    float x_step = 2*u_r / u_resolution.x;
    float y_step = 2*u_r / u_resolution.y;


    gl_Position = vec4((position.xy) * vec2(x_step, y_step) - vec2(1.0) + p, 0.0, 1.0);
    pos = offset;
    c = color;
}";

const TEXTURE_WIDTH: GLuint = 1080 * 2;
const TEXTURE_HEIGHT: GLuint = 1080;
const NUMBER_OF_PARTICLES: usize = 2000;
fn main() {
    let my_colors: Vec<Colour> = vec![
        Colour {
            tag: 0,
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        Colour {
            tag: 1,
            r: 0.0,
            g: 1.0,
            b: 0.0,
        },
        Colour {
            tag: 2,
            r: 0.0,
            g: 0.0,
            b: 1.0,
        },
    ];

    let mut forces = HashMap::new();
    forces.insert(
        (my_colors[0], my_colors[0]),
        Force {
            force: -0.01,
            d_repel: 5.0,
            d_colour: 100.0,
        },
    );

    forces.insert(
        (my_colors[1], my_colors[1]),
        Force {
            force: -0.01,
            d_repel: 5.0,
            d_colour: 100.0,
        },
    );

    forces.insert(
        (my_colors[0], my_colors[1]),
        Force {
            force: -0.1,
            d_repel: 5.0,
            d_colour: 100.0,
        },
    );

    forces.insert(
        (my_colors[1], my_colors[0]),
        Force {
            force: 0.1,
            d_repel: 5.0,
            d_colour: 100.0,
        },
    );

    let mut world = World::new((TEXTURE_WIDTH, TEXTURE_HEIGHT), forces, vec![]);

    unsafe {
        let (graphics, event_loop) = Graphics::new(TEXTURE_WIDTH, TEXTURE_HEIGHT, 1.0);
        let quad = Quad::new(
            Shader::new(&VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER).unwrap(),
            Shader::new(&FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER).unwrap(),
        )
        .unwrap();

        quad.program.apply();

        let mut positions = vec![];
        let mut colors: Vec<[f32; 4]> = vec![];
        let mut rng = thread_rng();
        for _ in 0..NUMBER_OF_PARTICLES {
            let col = rng.gen_range(0..=1);
            colors.push([(1.0 - col as f32), (0.0 + col as f32), 0.0, 1.0]);
            world.particles.push(Particle {
                position: Vec2 {
                    x: rng.gen_range(0.0..TEXTURE_WIDTH as f32),
                    y: rng.gen_range(0.0..TEXTURE_HEIGHT as f32),
                },
                velocity: Vec2 { x: 0.0, y: 1.0 },
                colour: my_colors[col],
                drag: 0.8,
            })
        }

        quad.program.set_float32_uniform("u_r", 2.0).unwrap();

        let offset_buffer = Buffer::new(gl::ARRAY_BUFFER);
        offset_buffer.set_data(&positions, gl::STATIC_DRAW);

        let offset_location = quad.program.get_attrib_location("offset").unwrap();
        quad.vertex_array.bind();
        quad.vertex_array
            .set_attribute::<[f32; 2]>(offset_location, 2, 0);
        gl::VertexAttribDivisor(offset_location, 1);

        let color_buffer = Buffer::new(gl::ARRAY_BUFFER);
        color_buffer.set_data(&positions, gl::STATIC_DRAW);

        let color_location = quad.program.get_attrib_location("color").unwrap();
        quad.vertex_array.bind();
        quad.vertex_array
            .set_attribute::<[f32; 4]>(color_location, 2, 0);
        gl::VertexAttribDivisor(color_location, 1);

        color_buffer.set_data(&colors, gl::STATIC_DRAW);

        graphics.window.set_title("Particle Life");

        let mut time = SystemTime::now();
        let mut mouse_pos = (0.0, 0.0);
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            let delta = time.elapsed().unwrap().as_secs_f64();

            if delta > 1.0 / 60.0 {
                world.simulation_step();
                positions = world
                    .particles
                    .clone()
                    .iter()
                    .map(|p| [p.position.x, p.position.y])
                    .collect();
                offset_buffer.set_data(&positions, gl::STATIC_DRAW);

                gl::ClearColor(0.0, 0.0, 0.0, 0.4);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                quad.draw_instanced(
                    (TEXTURE_WIDTH as f32, TEXTURE_HEIGHT as f32),
                    mouse_pos,
                    positions.len(),
                );
                graphics.swap_buffers();
                time = SystemTime::now();
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = (position.x as f32, TEXTURE_HEIGHT as f32 - position.y as f32);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
