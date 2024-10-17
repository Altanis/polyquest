use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use crate::fuzzy_compare;

#[derive(Default, Clone, Copy)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Vector2D<T> {
    /// Constructs a new vector from Cartesian (x, y) coordinates.
    pub fn new(x: T, y: T) -> Vector2D<T> {
        Vector2D { x, y }
    }

    /// Gets a vector from a scalar value.
    pub fn from_scalar(x: T) -> Vector2D<T> {
        Vector2D::new(x, x)
    }
}

impl<T: Copy> Add for Vector2D<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T> AddAssign for Vector2D<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Copy> Sub for Vector2D<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T> SubAssign for Vector2D<T>
where
    T: SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T> Mul<T> for Vector2D<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2D::new(self.x * rhs, self.y * rhs)
    }
}

impl<T> MulAssign<T> for Vector2D<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Vector2D<u32> {
    pub const INTEGER_ZERO: Vector2D<u32> = Vector2D { x: 0, y: 0 };

    /// Converts to a float vector.
    pub fn to_float(&self) -> Vector2D<f32> {
        Vector2D::new(self.x as f32, self.y as f32)
    }
}

impl Vector2D<f32> {
    pub const FLOAT_ZERO: Vector2D<f32> = Vector2D { x: 0.0, y: 0.0 };

    /// Constructs a new vector from polar (r, theta) coordinates.
    pub fn from_polar(radius: f32, theta: f32) -> Vector2D<f32> {
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

    /// Converts to an integer vector.
    pub fn to_integer(&self) -> Vector2D<u32> {
        Vector2D::new(self.x as u32, self.y as u32)
    }
}

// implement conversion of Vector2D<f32> to Vector2D<u32> (and vice versa)