use std::ops::{Add, AddAssign};
use crate::math::general::Scalar;
use crate::math::mat::Mat3x3;
impl<T> Add<T> for Mat3x3 where for<'a> &'a Mat3x3: Add<T, Output = Mat3x3> {
    type Output = Mat3x3;
    fn add(self, rhs: T) -> Self::Output {
        (&self) + rhs
    }
}
impl<T: Scalar> Add<T> for &Mat3x3 {
    type Output = Mat3x3;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Mat3x3::new(
            self.x + rhs,
            self.y + rhs,
            self.z + rhs,
        )
    }
}
impl Add<&Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;
    fn add(self, rhs: &Mat3x3) -> Self::Output {
        Mat3x3::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}

impl<T> AddAssign<T> for Mat3x3 where Mat3x3: Add<T, Output = Mat3x3> {
    fn add_assign(&mut self, rhs: T) {
        let new = *self + rhs;
        self.x = new.x;
        self.y = new.y;
        self.z = new.z;
    }
}