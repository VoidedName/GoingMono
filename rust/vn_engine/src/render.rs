use std::cmp::max;
use std::ops;
use winit::event::VirtualKeyCode::P;
use crate::color::RGBA;

impl ops::Add for PixelPosition {
    type Output = PixelPosition;
    fn add(self, rhs: Self) -> Self::Output {
        PixelPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub for PixelPosition {
    type Output = PixelPosition;
    fn sub(self, rhs: Self) -> Self::Output {
        PixelPosition {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

fn lerp(a: f64, b: f64, w: f64) -> f64 {
    a + ((b - a) * w)
}

//     *p1
//
//*p2      *p3
fn fill_flat_bottom<T: VNERenderer + ?Sized>(render: &mut T, p1: PixelPosition, p2: PixelPosition, p3: PixelPosition, color: RGBA) {
    let dxy_left = (p1.x as f64 - p2.x as f64) / (p1.y as f64 - p2.y as f64);
    let dxy_right = (p1.x as f64 - p3.x as f64) / (p1.y as f64 - p3.y as f64);

    let mut xl = p1.x as f64;
    let mut xr = p1.x as f64;

    for y in p1.y..p2.y {
        render.draw_line(PixelPosition { x: xl.floor() as u32, y },
                         PixelPosition { x: xr.ceil() as u32, y }, color);
        xl += dxy_left;
        xr += dxy_right;
    }
}


//*p1       *p2
//
//     *p3
fn fill_flat_top<T: VNERenderer + ?Sized>(render: &mut T, p1: PixelPosition, p2: PixelPosition, p3: PixelPosition, color: RGBA) {
    let dxy_left = (p1.x as f64 - p3.x as f64) / (p1.y as f64 - p3.y as f64);
    let dxy_right = (p2.x as f64 - p3.x as f64) / (p2.y as f64 - p3.y as f64);

    let mut xl = p3.x as f64;
    let mut xr = p3.x as f64;

    for y in (p1.y..p3.y).rev() {
        render.draw_line(PixelPosition { x: xl.floor() as u32, y },
                         PixelPosition { x: xr.ceil() as u32, y }, color);
        xl -= dxy_left;
        xr -= dxy_right;
    }
}

pub trait VNERenderer {
    fn set_title(&mut self, title: &str);
    fn clear_screen(&mut self, color: RGBA);
    fn draw_pixel(&mut self, position: PixelPosition, color: RGBA);
    fn draw_line(&mut self, from: PixelPosition, to: PixelPosition, color: RGBA) {
        let dx = to.x as i32 - from.x as i32;
        let dy = to.y as i32 - from.y as i32;

        let steps = dx.abs().max(dy.abs());

        let x_inc = dx as f64 / steps as f64;
        let y_inc = dy as f64 / steps as f64;

        let mut x = from.x as f64;
        let mut y = from.y as f64;
        self.draw_pixel(PixelPosition { x: x as u32, y: y as u32 }, color);
        for _ in 0..steps as i32 {
            x += x_inc;
            y += y_inc;
            self.draw_pixel(PixelPosition { x: x.round() as u32, y: y.round() as u32 }, color);
        }
    }
    fn fill_triangle(&mut self, v1: PixelPosition, v2: PixelPosition, v3: PixelPosition, color: RGBA) {
        let mut sorted = vec![v1, v2, v3];
        sorted.sort_by(|a, b| {
            let cmp = a.y.partial_cmp(&b.y).unwrap();
            if cmp.is_eq() {
                a.x.partial_cmp(&b.x).unwrap()
            } else {
                cmp
            }
        });
        let v1 = sorted[0];
        let v2 = sorted[1];
        let v3 = sorted[2];

        if v2.y == v3.y {
            fill_flat_bottom(self, v1, v2, v3, color);
            return;
        }
        if v1.y == v2.y {
            fill_flat_top(self, v1, v2, v3, color);
            return;
        }

        let long = v3.y - v1.y;
        let short = v3.y - v2.y;
        let weight = short as f64 / long as f64;
        let v4 = PixelPosition {
            x: (lerp(v3.x as f64, v1.x as f64, weight) + 0.5) as u32,
            y: v2.y,
        };

        if v2.x < v4.x {
            fill_flat_bottom(self, v1, v2, v4, color);
            fill_flat_top(self, v2, v4, v3, color);
        } else {
            fill_flat_bottom(self, v1, v4, v2, color);
            fill_flat_top(self, v4, v2, v3, color);
        }
    }
}

pub trait VNERendererCommit {
    /// commit all drawing operations
    fn commit(&mut self);
}

#[derive(Copy, Clone, Debug)]
pub struct PixelPosition {
    pub x: u32,
    pub y: u32,
}
