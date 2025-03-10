mod mul;
mod div;
mod add;
mod sub;
mod iter_ops;

use std::fmt::Display;
use std::ops::Neg;

/// A 3-dimensional Vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    /// The x-component of the vector
    pub x: f64,
    /// The y-component of the vector
    pub y: f64,
    /// The z-component of the vector
    pub z: f64,
}
impl Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Vector3 {
    /// creates a Vector with random components (range 0 to 1)
    pub fn random() -> Self {
        Self {
            x: fastrand::f64(),
            y: fastrand::f64(),
            z: fastrand::f64(),
        }
    }
    pub fn random_direction() -> Self {
        let z = fastrand::f64() * 2.0 - 1.0;
        let theta = fastrand::f64() * 2.0 * std::f64::consts::PI;
        let r = (1.0 - z * z).sqrt();
        Self {
            x: r * theta.cos(),
            y: r * theta.sin(),
            z,
        }.norm()
    }
    /// creates a Vector with all ones
    pub const fn ones() -> Vector3 {
        Vector3 {
            x: 1.,
            y: 1.,
            z: 1.,
        }
    }
    /// creates a unit vector along the x-Axis
    pub const fn x() -> Vector3 {
        Self::const_new(1.0, 0.0, 0.0)
    }
    /// creates a unit vector along the y-Axis
    pub const fn y() -> Vector3 {
        Self::const_new(0.0, 1.0, 0.0)
    }
    /// creates a unit vector along the z-Axis
    pub const fn z() -> Vector3 {
        Self::const_new(0.0, 0.0, 1.0)
    }
    /// creates a new vector with the given x, y and z components
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Vector3 {
        Vector3 { x: x.into(), y: y.into(), z:z.into() }
    }
    /// constant function for creating a new Vector
    pub const fn const_new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x, y, z }
    }
    /// updates the vector to a new value
    pub fn update(&mut self, new: Vector3) -> () {
        self.x = new.x;
        self.y = new.y;
        self.z = new.z;
    }
    /// creates a new vector with all zeros
    pub const fn zeros() -> Self {
        Vector3::const_new(0.0, 0.0, 0.0)
    }
    /// calculates the dot product between two vectors
    pub fn dot(&self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    /// calculates the cross product between two vecotrs
    pub fn cross(&self, other: Self) -> Self {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    /// returns the sum of the components
    pub fn sum(&self) -> f64 {
        self.x + self.y + self.z
    }
    /// returns the length of the vector
    pub fn len(&self) -> f64 {
        (self * self).sum().sqrt()
    }
    /// returns the normalized vector (same direction but length = 1)
    pub fn norm(&self) -> Self {
        self / self.len()
    }
}
impl<A: Into<f64>, B: Into<f64>, C: Into<f64>> From<(A, B, C)> for Vector3 {
    fn from(tuple: (A, B, C)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}
impl<T: Into<f64> + Clone> From<[T; 3]> for Vector3 where {
    fn from(array: [T; 3]) -> Self {
        Self::new(Into::<f64>::into(array[0].clone()), Into::<f64>::into(array[1].clone()), Into::<f64>::into(array[2].clone()))
    }
}
impl From<&Vector3> for Vector3 {
    fn from(vector: &Vector3) -> Self {
        Self::new(vector.x, vector.y, vector.z)
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}
impl Neg for &Vector3 {
    type Output = Vector3;
    fn neg(self) -> Self::Output {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}