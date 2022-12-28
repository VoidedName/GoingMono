use super::render::VNERenderer;
use crate::opengl::OpenGLRenderer;
use crate::render::{VNEFullRenderer, VNERendererCommit, VNERendererWindow};
use crate::sprite::SpriteBased;
use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use std::time::Instant;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

pub struct VNEngine<T: VNEFullRenderer> {
    event_loop: EventLoop<()>,
    engine_state: VNEngineState,
    renderer: T,
}

pub struct VNEngineState {
    pub width: u32,
    pub height: u32,
    pub delta: f64,
    pub focused: bool,
    device_state: DeviceState,
    inner_mouse_pos: (i32, i32),
}

impl VNEngineState {
    pub fn mouse(&self) -> MouseState {
        let state = self.device_state.get_mouse();
        let (x, y) = state.coords;
        let (ix, iy) = self.inner_mouse_pos;
        MouseState {
            coords: (x - ix, y - iy),
            button_pressed: state.button_pressed,
        }
    }

    pub fn keyboad(&self) -> Vec<Keycode> {
        self.device_state.get_keys()
    }
}

impl VNEngine<OpenGLRenderer> {
    pub fn new_opengl(width: u32, height: u32, scale: u32) -> Box<VNEngine<OpenGLRenderer>> {
        let event_loop = EventLoop::new();
        let mut renderer = OpenGLRenderer::new(width, height, scale, &event_loop);
        Box::new(VNEngine {
            renderer,
            engine_state: VNEngineState {
                width,
                height,
                delta: 0.0,
                device_state: DeviceState::new(),
                focused: false,
                inner_mouse_pos: (0, 0),
            },
            event_loop,
        })
    }
}

impl VNEngine<SpriteBased> {
    pub fn new_sprite_based(width: u32, height: u32, scale: u32) -> Box<VNEngine<SpriteBased>> {
        let event_loop = EventLoop::new();
        let mut renderer = SpriteBased::new(width as usize, height as usize, scale, &event_loop);
        Box::new(VNEngine {
            renderer,
            engine_state: VNEngineState {
                width,
                height,
                delta: 0.0,
                device_state: DeviceState::new(),
                focused: false,
                inner_mouse_pos: (0, 0),
            },
            event_loop,
        })
    }
}

impl<T: VNEFullRenderer> VNEngine<T> {
    pub fn new(width: u32, height: u32, renderer: T) -> Box<VNEngine<T>> {
        let event_loop = EventLoop::new();
        Box::new(VNEngine {
            event_loop,
            renderer,
            engine_state: VNEngineState {
                focused: false,
                delta: 0.0,
                width,
                height,
                device_state: DeviceState::new(),
                inner_mouse_pos: (0, 0),
            },
        })
    }

    pub fn engine_state(&self) -> &VNEngineState {
        &self.engine_state
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
                    if delta > 0 && self.engine_state.focused {
                        let inner_pos = renderer.window().inner_position().unwrap();
                        self.engine_state.inner_mouse_pos = (inner_pos.x, inner_pos.y);
                        runner.tick(&self.engine_state, renderer);
                        renderer.commit();
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(in_focus),
                    window_id,
                } => {
                    self.engine_state.focused = in_focus;
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } => {
                    runner.tear_down(&self.engine_state, renderer);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            }
        });
    }
}

pub trait VNERunner {
    /// Do your setup code here
    fn setup(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {}
    /// Gets called every frame
    fn tick(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized));
    /// do your teadown code here
    fn tear_down(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {}
}
