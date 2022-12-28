use crate::color::RGBA;
use std::cmp::max;
use std::ops;
use winit::event::VirtualKeyCode::P;
use winit::window::Window;

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
fn fill_flat_bottom<T: VNERenderer + ?Sized>(
    render: &mut T,
    p1: PixelPosition,
    p2: PixelPosition,
    p3: PixelPosition,
    color: RGBA,
) {
    let dxy_left = (p1.x as f64 - p2.x as f64) / (p1.y as f64 - p2.y as f64);
    let dxy_right = (p1.x as f64 - p3.x as f64) / (p1.y as f64 - p3.y as f64);

    let mut xl = p1.x as f64;
    let mut xr = p1.x as f64;

    for y in p1.y..p2.y {
        render.draw_line(
            PixelPosition {
                x: xl.floor() as u32,
                y,
            },
            PixelPosition {
                x: xr.ceil() as u32,
                y,
            },
            color,
        );
        xl += dxy_left;
        xr += dxy_right;
    }
}

//*p1       *p2
//
//     *p3
fn fill_flat_top<T: VNERenderer + ?Sized>(
    render: &mut T,
    p1: PixelPosition,
    p2: PixelPosition,
    p3: PixelPosition,
    color: RGBA,
) {
    let dxy_left = (p1.x as f64 - p3.x as f64) / (p1.y as f64 - p3.y as f64);
    let dxy_right = (p2.x as f64 - p3.x as f64) / (p2.y as f64 - p3.y as f64);

    let mut xl = p3.x as f64;
    let mut xr = p3.x as f64;

    for y in (p1.y..p3.y).rev() {
        render.draw_line(
            PixelPosition {
                x: xl.floor() as u32,
                y,
            },
            PixelPosition {
                x: xr.ceil() as u32,
                y,
            },
            color,
        );
        xl -= dxy_left;
        xr -= dxy_right;
    }
}

/// test if point point is to the right of vector v1 -> v2
fn edge_function(v1: PixelPosition, v2: PixelPosition, point: PixelPosition) -> bool {
    (point.x as i32 - v1.x as i32) * (v2.y as i32 - v1.y as i32) - (point.y as i32 - v1.y as i32) * (v2.x as i32 - v1.x as i32) >= 0
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
        self.draw_pixel(
            PixelPosition {
                x: x as u32,
                y: y as u32,
            },
            color,
        );
        for _ in 0..steps as i32 {
            x += x_inc;
            y += y_inc;
            self.draw_pixel(
                PixelPosition {
                    x: x.round() as u32,
                    y: y.round() as u32,
                },
                color,
            );
        }
    }


    /// assumes clockwise vertex order
    fn fill_triangle(
        &mut self,
        v1: PixelPosition,
        v2: PixelPosition,
        v3: PixelPosition,
        clamp_top_left: PixelPosition,
        clamp_bottom_right: PixelPosition,
        color: RGBA,
    ) {
        let x_min = v1.x.min(v2.x).min(v3.x).max(clamp_top_left.x);
        let x_max = v1.x.max(v2.x).max(v3.x).min(clamp_bottom_right.x);
        let y_min = v1.y.min(v2.y).min(v3.y).max(clamp_top_left.y);
        let y_max = v1.y.max(v2.y).max(v3.y).min(clamp_bottom_right.y);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let point = PixelPosition{ x, y };
                let mut inside = edge_function(v1, v2, point);
                inside &= edge_function(v2, v3, point);
                inside &= edge_function(v3, v1, point);
                if inside {
                    self.draw_pixel(point, color);
                }
            }
        }
    }
}

pub trait VNERendererCommit {
    /// commit all drawing operations
    fn commit(&mut self);
}

pub trait VNERendererWindow {
    fn window(&mut self) -> &Window;
}

pub trait VNEFullRenderer: VNERenderer + VNERendererWindow + VNERendererCommit {}

#[derive(Copy, Clone, Debug)]
pub struct PixelPosition {
    pub x: u32,
    pub y: u32,
}
