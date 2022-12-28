use vn_engine::color::{GREEN, WHITE};
use vn_engine::engine::{VNEngineState, VNEngine, VNERunner};
use vn_engine::opengl::OpenGLRenderer;
use vn_engine::render::{PixelPosition, VNERenderer};

struct State {}

impl VNERunner for State {
    fn tick(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        // println!("{}", 1.0 / engine.delta);
        println!("{:?}", engine.keyboad());
        for x in 0..engine.width {
            for y in 0..engine.height {
                renderer.draw_pixel( PixelPosition{x, y}, GREEN);
            }
        }
    }
}

fn main() {
    let mut state = State {};
    let mut engine = VNEngine::new_opengl(600, 400, 4);
    engine.run(&mut state);
}
