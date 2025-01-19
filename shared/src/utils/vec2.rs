use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::{fuzzy_compare, lerp, rand};
use rand::Rng;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    /// Constructs a new vector from Cartesian (x, y) coordinates.
    pub fn new(x: f32, y: f32) -> Vector2D {
        Vector2D { x, y }
    }

    /// Gets a vector from a scalar value.
    pub fn from_scalar(x: f32) -> Vector2D {
        Vector2D::new(x, x)
    }
}

impl Neg for Vector2D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vector2D::new(-self.x, -self.y)
    }
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

    /// Constructs a new vector from polar (r, theta) coordinates.
    pub fn from_polar(radius: f32, theta: f32) -> Vector2D {
        Vector2D::new(radius * theta.cos(), radius * theta.sin())
    }

    /// Whether or not the vector is zero.
    pub fn is_zero(&self, tolerance: f32) -> bool {
        self.partial_eq(Vector2D::ZERO, tolerance)
    }

    /// Gets the angle from the x-axis.
    pub fn angle(&self) -> f32 {
        f32::atan2(self.y, self.x)
    }

    /// Lerps a vector towards another one.
    pub fn lerp_towards(&mut self, other: Vector2D, factor: f32) {
        self.x = lerp!(self.x, other.x, factor);
        self.y = lerp!(self.y, other.y, factor);
    }

    /// Creates a random vector in bounds of [min, max].
    pub fn from_random(min: f32, max: f32) -> Vector2D {
        Vector2D::new(rand!(min, max), rand!(min, max))
    }

    /// Gets the magnitude^2 of the vector.
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Gets the magnitude of the vector.
    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }    

    /// Gets the distance of the vector from another vector.
    pub fn distance(self, other: Vector2D) -> f32 {
        (other - self).magnitude()
    }

    /// Gets the distance squared of the vector from another vector.
    pub fn distance_squared(self, other: Vector2D) -> f32 {
        (other - self).magnitude_squared()
    }

    /// Checks if two vectors are nearly equivalent.
    pub fn partial_eq(self, other: Vector2D, tolerance: f32) -> bool {
        fuzzy_compare!(self.x, other.x, tolerance) && fuzzy_compare!(self.y, other.y, tolerance)   
    }

    /// Returns the smaller component.
    pub fn min(&self) -> f32 {
        self.x.min(self.y)
    }

    /// Normalizes the vector to have a magnitude of 1 (unit vector).
    /// If the vector's magnitude is 0, it does nothing.
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            *self *= 1.0 / mag;
        }
    }

    /// Sets the vector's magnitude to the specified value.
    pub fn set_magnitude(&mut self, magnitude: f32) {
        self.normalize();
        *self *= magnitude;
    }

    /// Constrains the components of the vector.
    pub fn constrain(&mut self, min: f32, max: f32) {
        self.x = self.x.min(max).max(min);
        self.y = self.y.min(max).max(min);
    }

    /// Swaps the vectors components.
    pub fn swap(&mut self) -> &mut Self {
        std::mem::swap(&mut self.x, &mut self.y);
        self
    }
}