use crate::opengl::Graphics;
use crate::particle_life::Force;
use crate::utils::Vec2;
use gl::types::GLuint;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod opengl;
mod particle_life;
mod utils;

const TEXTURE_WIDTH: GLuint = 1080;
const TEXTURE_HEIGHT: GLuint = 1080;
fn main() {
    unsafe {
        let (graphics, event_loop) = Graphics::new(TEXTURE_WIDTH, TEXTURE_HEIGHT, 1.0);
        graphics.window.set_title("Particle Life");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            graphics.swap_buffers();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
