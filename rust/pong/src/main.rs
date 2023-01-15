use std::ops::Neg;
use vn_engine::color::{BLACK, BLUE, GREEN, RED, VIOLET, WHITE};
use vn_engine::engine::{VNEngineState, VNEngine, VNERunner};
use vn_engine::Keycode;
use vn_engine::opengl::OpenGLRenderer;
use vn_engine::render::{PixelPosition, VNERenderer};

struct State {
    ball_pos: [f64; 2],
    left_paddle: f64,
    right_paddle: f64,
    ball_velocity: [f64; 2],
    stop: bool,
}

impl VNERunner for State {
    fn setup(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        renderer.set_title(format!("Pong! FPS: {}", 1.0 / engine.delta).as_str());
    }

    fn tick(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        const BALL_RADIUS: u32 = 2;
        const PADDLE_HEIGHT: u32 = 50;
        const PADDLE_SPEED: f64 = 150.0;
        const SPEED_INCREASE: f64 = 10.0;

        renderer.clear_screen(BLACK);
        renderer.set_title(format!("Pong! FPS: {}", 1.0 / engine.delta).as_str());

        let left_edge = 25;
        let right_edge = engine.width - 25;
        let top_edge = 25;
        let bottom_edge = engine.height - 25;

        // draw field
        renderer.fill_rectangle(PixelPosition { x: left_edge, y: top_edge }, PixelPosition { x: right_edge, y: bottom_edge }, WHITE);
        renderer.draw_rectangle(PixelPosition { x: left_edge, y: top_edge }, PixelPosition { x: right_edge, y: bottom_edge }, VIOLET);

        // draw paddles
        renderer.fill_rectangle(
            PixelPosition { x: left_edge - BALL_RADIUS, y: self.left_paddle as u32 - PADDLE_HEIGHT / 2},
            PixelPosition { x: left_edge + BALL_RADIUS, y: self.left_paddle as u32 + PADDLE_HEIGHT / 2},
            GREEN,
        );
        renderer.fill_rectangle(
            PixelPosition { x: right_edge - BALL_RADIUS, y: self.right_paddle as u32 - PADDLE_HEIGHT / 2},
            PixelPosition { x: right_edge + BALL_RADIUS, y: self.right_paddle as u32 + PADDLE_HEIGHT / 2},
            GREEN,
        );

        // language = draw ball
        let [x, y] = self.ball_pos;
        let [x, y] = [x as u32, y as u32];
        renderer.fill_rectangle(PixelPosition { x: x - BALL_RADIUS, y: y - BALL_RADIUS }, PixelPosition { x: x + BALL_RADIUS , y: y + BALL_RADIUS }, RED);

        // updates
        if !self.stop {
            // update ball position
            self.ball_pos[0] += self.ball_velocity[0] * engine.delta;
            self.ball_pos[1] += self.ball_velocity[1] * engine.delta;

            // update paddle positions
            if engine.keyboad().contains(&Keycode::Up) {
                self.right_paddle -= PADDLE_SPEED * engine.delta;
            }
            if engine.keyboad().contains(&Keycode::Down) {
                self.right_paddle += PADDLE_SPEED * engine.delta;
            }

            // update paddle positions
            if engine.keyboad().contains(&Keycode::W) {
                self.left_paddle -= PADDLE_SPEED * engine.delta;
            }
            if engine.keyboad().contains(&Keycode::S) {
                self.left_paddle += PADDLE_SPEED * engine.delta;
            }

            // bounce bottom
            if y + BALL_RADIUS >= bottom_edge {
                self.ball_velocity[1] = -self.ball_velocity[1].abs();
            }

            // bounce top
            if y - BALL_RADIUS <= top_edge {
                self.ball_velocity[1] = self.ball_velocity[1].abs();
            }

            // bounce right
            if x + BALL_RADIUS >= right_edge - BALL_RADIUS && y <= self.right_paddle as u32 + PADDLE_HEIGHT / 2 && y >= self.right_paddle as u32 - PADDLE_HEIGHT / 2 {
                self.ball_velocity[0] = -self.ball_velocity[0].abs();
                self.ball_pos[0] = right_edge as f64 - BALL_RADIUS as f64 * 2.0 - 1.0;
            }

            // bounce left
            if x - BALL_RADIUS <= left_edge + BALL_RADIUS && y <= self.left_paddle as u32 + PADDLE_HEIGHT / 2 && y >= self.left_paddle as u32 - PADDLE_HEIGHT / 2 {
                self.ball_velocity[0] = self.ball_velocity[0].abs();
                self.ball_pos[0] = left_edge as f64 + BALL_RADIUS as f64 * 2.0 + 1.0;
            }

            // game over
            if self.ball_pos[0] as u32 + BALL_RADIUS > right_edge || self.ball_pos[0] as u32 - BALL_RADIUS < left_edge {
                println!("{}", self.ball_pos[0]);
                self.stop = true;
            }

            // accelerate ball
            let ratio = self.ball_velocity[0] / (self.ball_velocity[0] + self.ball_velocity[1]);
            self.ball_velocity[0] += ratio * self.ball_velocity[0].signum() * SPEED_INCREASE * engine.delta;
            self.ball_velocity[1] += (1.0 - ratio) * self.ball_velocity[1].signum() * SPEED_INCREASE * engine.delta;
        }
    }
}

fn main() {
    let mut state = State {
        right_paddle: 200.0,
        left_paddle: 200.0,
        ball_pos: [300.0, 200.0],
        ball_velocity: [100.0, 20.0],
        stop: false,
    };
    let mut engine = VNEngine::new_opengl(600, 400, 4);
    engine.run(&mut state);
}
