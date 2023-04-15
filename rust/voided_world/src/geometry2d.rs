use std::ops::{Add, Div, Mul, Rem, Sub};

// Vector / Point

pub trait Coordinate<T = Self>:
    Copy
    + Clone
    + PartialOrd
    + PartialEq
    + Sub<Output = T>
    + Mul<Output = T>
    + Add<Output = T>
    + Div<Output = T>
    + Rem<Output = T>
{
}

impl<
        T: Copy
            + Clone
            + PartialOrd
            + PartialEq
            + Sub<Output = T>
            + Mul<Output = T>
            + Add<Output = T>
            + Div<Output = T>
            + Rem<Output = T>,
    > Coordinate for T
{
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vector2D<T: Coordinate> {
    pub(crate) x: T,
    pub(crate) y: T,
}

pub type Point<T> = Vector2D<T>;

impl<T: Coordinate> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Coordinate> Add for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Coordinate> Sub for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Coordinate> Mul for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl<T: Coordinate> Div for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<T: Coordinate> Rem for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x % rhs.x, self.y % rhs.y)
    }
}

impl Into<Vector2D<f64>> for Vector2D<f32> {
    fn into(self) -> Vector2D<f64> {
        Vector2D::new(self.x as f64, self.y as f64)
    }
}

impl Into<Vector2D<f64>> for &Vector2D<f32> {
    fn into(self) -> Vector2D<f64> {
        Vector2D::new(self.x as f64, self.y as f64)
    }
}

impl Into<Vector2D<f32>> for Vector2D<f64> {
    fn into(self) -> Vector2D<f32> {
        Vector2D::new(self.x as f32, self.y as f32)
    }
}

impl Into<Vector2D<f32>> for &Vector2D<f64> {
    fn into(self) -> Vector2D<f32> {
        Vector2D::new(self.x as f32, self.y as f32)
    }
}

// Rectangle

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Rectangle<T: Coordinate> {
    pub low: Point<T>,
    pub high: Point<T>,
}

pub enum RectangleContainsParameter<T: Coordinate> {
    Point(Point<T>),
    Rectangle(Rectangle<T>),
}

impl<T: Coordinate> Rectangle<T> {
    pub fn intersects(&self, other: &Rectangle<T>) -> bool {
        self.low.x <= other.high.x
            && self.high.x >= other.low.x
            && self.low.y <= other.high.y
            && self.high.y >= other.low.y
    }

    pub fn contains(&self, target: &RectangleContainsParameter<T>) -> bool {
        match target {
            RectangleContainsParameter::Rectangle(rectangle) => {
                self.low.x <= rectangle.low.x
                    && self.high.x >= rectangle.high.x
                    && self.low.y <= rectangle.low.y
                    && self.high.y >= rectangle.high.y
            }
            RectangleContainsParameter::Point(point) => {
                self.low.x <= point.x
                    && self.high.x >= point.x
                    && self.low.y <= point.y
                    && self.high.y >= point.y
            }
        }
    }

    pub fn merge(&self, other: &Rectangle<T>) -> Self {
        let low_x = if self.low.x < other.low.x {
            self.low.x
        } else {
            other.low.x
        };
        let high_x = if self.high.x > other.high.x {
            self.high.x
        } else {
            other.high.x
        };
        let low_y = if self.low.y < other.low.y {
            self.low.y
        } else {
            other.low.y
        };
        let high_y = if self.high.y > other.high.y {
            self.high.y
        } else {
            other.high.y
        };

        Self {
            low: Vector2D::new(low_x, low_y),
            high: Vector2D::new(high_x, high_y),
        }
    }

    pub fn area(&self) -> T {
        let w = self.high.x - self.low.x;
        let h = self.high.y - self.low.y;
        w * h
    }
}

pub trait Spacial<T: Coordinate>: Clone {
    fn boundary(&self) -> &Rectangle<T>;
}
