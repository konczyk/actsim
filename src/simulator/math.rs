use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2D {
    x: f64,
    y: f64
}

impl Vector2D {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: Self) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

impl Add for Vector2D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}
impl Sub for Vector2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f64> for Vector2D {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}
