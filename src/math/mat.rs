mod add;
mod sub;
mod mul;
mod div;
mod specific_math;

use std::fmt::{Debug, Display, Formatter};
use crate::math::vector::Vector3;

#[derive(Clone, Copy)]
pub struct Mat3x3 {
    /// The first row of the matrix
    pub x: Vector3,
    /// The second row of the matrix
    pub y: Vector3,
    /// The third row of the matrix
    pub z: Vector3,
}

impl Display for Mat3x3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}\n {}\n {}]", self.x, self.y, self.z)
    }
}
impl Debug for Mat3x3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}\n {}\n {}]", self.x, self.y, self.z)
    }
}