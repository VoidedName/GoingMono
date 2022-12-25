use super::render::VNERenderer;
use crate::opengl::OpenGLRenderer;
use crate::render::VNERendererCommit;
use std::time::Instant;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct VNEngine<R: VNERenderer + VNERendererCommit> {
    event_loop: EventLoop<()>,
    renderer: R,
    engine_state: EngineState,
    focused: bool,
}

pub struct EngineState {
    pub width: u32,
    pub height: u32,
    pub delta: f64,
}

impl VNEngine<OpenGLRenderer> {
    pub fn new_opengl(width: u32, height: u32, scale: u32) -> VNEngine<OpenGLRenderer> {
        let event_loop = EventLoop::new();
        VNEngine {
            engine_state: EngineState {
                width,
                height,
                delta: 0.0,
            },
            focused: false,
            renderer: OpenGLRenderer::new(width, height, scale, &event_loop),
            event_loop,
        }
    }

    pub fn engine_state(&self) -> &EngineState {
        &self.engine_state
    }
}

impl<T: VNERenderer + VNERendererCommit> VNEngine<T> {
    pub fn new(width: u32, height: u32, renderer: T) -> VNEngine<T> {
        let event_loop = EventLoop::new();
        VNEngine {
            renderer,
            event_loop,
            focused: false,
            engine_state: EngineState {
                delta: 0.0,
                width,
                height,
            },
        }
    }

    pub fn run(&mut self, runner: &mut impl VNERunner) {
        let mut previous_frame_time = Instant::now();

        let mut event_loop = &mut self.event_loop;
        let renderer = &mut self.renderer;

        runner.setup(&self.engine_state, renderer);

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    let delta = previous_frame_time.elapsed().as_nanos();
                    self.engine_state.delta = delta as f64 / 1_000_000_000.0;

                    previous_frame_time = Instant::now();
                    if delta > 0 && self.focused {
                        runner.tick(&self.engine_state, renderer);
                        renderer.commit();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Focused(in_focus), window_id } => {
                    self.focused = in_focus;
                    println!("in focus: {}", in_focus);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } => {
                    runner.tear_down(&self.engine_state, renderer);
                    *control_flow = ControlFlow::Exit
                },
                _ => (),
            }
        });
    }
}

pub trait VNERunner {
    /// Do your setup code here
    fn setup(&mut self, engine: &EngineState, renderer: &mut impl VNERenderer) {}
    /// Gets called every frame
    fn tick(&mut self, engine: &EngineState, renderer: &mut impl VNERenderer);
    /// do your teadown code here
    fn tear_down(&mut self, engine: &EngineState, renderer: &mut impl VNERenderer) {}
}
