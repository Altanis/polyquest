use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::{fuzzy_compare, lerp, rand};
use rand::Rng;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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

impl<T: Copy> Neg for Vector2D<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vector2D::new(-self.x, -self.y)
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

impl Vector2D<f32> {
    pub const ZERO: Vector2D<f32> = Vector2D { x: 0.0, y: 0.0 };

    /// Constructs a new vector from polar (r, theta) coordinates.
    pub fn from_polar(radius: f32, theta: f32) -> Vector2D<f32> {
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
    pub fn lerp_towards(&mut self, other: Vector2D<f32>, factor: f32) {
        self.x = lerp!(self.x, other.x, factor);
        self.y = lerp!(self.y, other.y, factor);
    }

    /// Creates a random vector in bounds of [min, max].
    pub fn from_random(min: f32, max: f32) -> Vector2D<f32> {
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
    pub fn distance(self, other: Vector2D<f32>) -> f32 {
        (other - self).magnitude()
    }

    /// Gets the distance squared of the vector from another vector.
    pub fn distance_squared(self, other: Vector2D<f32>) -> f32 {
        (other - self).magnitude_squared()
    }

    /// Checks if two vectors are nearly equivalent.
    pub fn partial_eq(self, other: Vector2D<f32>, tolerance: f32) -> bool {
        fuzzy_compare!(self.x, other.x, tolerance) && fuzzy_compare!(self.y, other.y, tolerance)   
    }

    /// Returns the smaller component.
    pub fn min(&self) -> f32 {
        self.x.min(self.y)
    }
}