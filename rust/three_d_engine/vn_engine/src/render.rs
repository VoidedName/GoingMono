use std::cmp::max;
use crate::color::RGBA;

pub trait VNERenderer {
    fn set_title(&mut self, title: &str);
    fn clear_screen(&mut self, color: RGBA);
    fn draw_pixel(&mut self, position: Position, color: RGBA);
    fn draw_line(&mut self, from: Position, to: Position, color: RGBA) {
        let dx = to.x as i32 - from.x as i32;
        let dy = to.y as i32 - from.y as i32;

        let steps = max(dx.abs(), dy.abs());

        let x_inc = dx as f32 / steps as f32;
        let y_inc = dy as f32 / steps as f32;

        let mut x = from.x as f32;
        let mut y = from.y as f32;
        for step in 0..steps {
            x += x_inc;
            y += y_inc;
            self.draw_pixel(Position { x: x.round() as u32, y: y.round() as u32 }, color);
        }
    }
}

pub trait VNERendererCommit {
    /// commit all drawing operations
    fn commit(&mut self);
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}
