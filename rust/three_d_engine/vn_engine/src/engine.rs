use super::render::VNERenderer;
use crate::opengl::OpenGLRenderer;
use crate::render::VNERendererCommit;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct VNEngine<R: VNERenderer + VNERendererCommit> {
    pub width: u32,
    pub height: u32,
    event_loop: EventLoop<()>,
    renderer: R,
}



impl VNEngine<OpenGLRenderer> {
    pub fn new_opengl(width: u32, height: u32, scale: u32) -> VNEngine<OpenGLRenderer> {
        let event_loop = EventLoop::new();
        VNEngine {
            width,
            height,
            renderer: OpenGLRenderer::new(width, height, scale, &event_loop),
            event_loop,
        }
    }
}

impl<T: VNERenderer + VNERendererCommit> VNEngine<T> {
    pub fn new(width: u32, height: u32, renderer: T) -> VNEngine<T> {
        let event_loop = EventLoop::new();
        VNEngine {
            renderer,
            event_loop,
            width,
            height,
        }
    }

    pub fn run(&mut self, runner: &mut impl VNERunner) {
        let mut previous_frame_time = Instant::now();

        let mut event_loop = &mut self.event_loop;
        let renderer = &mut self.renderer;

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    let delta = previous_frame_time.elapsed().as_nanos();
                    previous_frame_time = Instant::now();
                    if delta == 0 {
                        return;
                    }
                    runner.tick(delta, renderer);
                    renderer.commit();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}

pub trait VNERunner {
    /// Do your setup code here
    fn setup(&mut self) {}
    /// Gets called every frame
    fn tick(&mut self, nano_delta: u128, renderer: &mut impl VNERenderer);
    /// do your teadown code here
    fn tear_down(&mut self) {}
}
