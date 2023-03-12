use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Sub};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Sub for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Sub<f32> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Add for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Add<f32> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Mul<f32> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
impl Div<f32> for &Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Vec2 {
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let magnitude = self.len();
        self / magnitude
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct Colour {
    pub tag: u32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl Eq for Colour {}
impl Hash for Colour {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
    }
}
