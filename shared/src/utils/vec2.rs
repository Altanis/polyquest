use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use crate::fuzzy_compare;

#[derive(Default, Clone, Copy)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32
}

impl Add for Vector2D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vector2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vector2D {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vector2D {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;   
    }
}

impl Mul<f32> for Vector2D {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector2D::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f32> for Vector2D {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Vector2D {
    pub const ZERO: Vector2D = Vector2D { x: 0.0, y: 0.0 };
    
    /// Constructs a new vector from Cartesian (x, y) coordinates.
    pub fn new(x: f32, y: f32) -> Vector2D {
        Vector2D { x, y }
    }

    /// Constructs a new vector from polar (r, theta) coordinates.
    pub fn from_polar(radius: f32, theta: f32) -> Vector2D {
        Vector2D::new(radius * theta.cos(), radius * theta.sin())
    }

    /// Whether or not the vector is zero.
    pub fn is_zero(&self, tolerance: f32) -> bool {
        fuzzy_compare!(self.x, 0.0, tolerance) && fuzzy_compare!(self.y, 0.0, tolerance)
    }

    /// Gets the angle from the x-axis.
    pub fn angle(&self) -> f32 {
        f32::atan2(self.y, self.x)
    }
}