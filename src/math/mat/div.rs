use crate::math::general::Scalar;
use crate::math::mat::Mat3x3;
use std::ops::{Div, DivAssign};
impl<T> Div<T> for Mat3x3 where for<'a> &'a Mat3x3: Div<T, Output = Mat3x3> {
    type Output = Mat3x3;
    fn div(self, rhs: T) -> Self::Output {
        (&self) / rhs
    }
}
impl<T: Scalar> Div<T> for &Mat3x3 {
    type Output = Mat3x3;
    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Mat3x3::new(
            self.x / rhs,
            self.y / rhs,
            self.z / rhs,
        )
    }
}
impl Div<&Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;
    fn div(self, rhs: &Mat3x3) -> Self::Output {
        Mat3x3::new(
            self.x / rhs.x,
            self.y / rhs.y,
            self.z / rhs.z,
        )
    }
}

impl<T> DivAssign<T> for Mat3x3 where Mat3x3: Div<T, Output = Mat3x3> {
    fn div_assign(&mut self, rhs: T) {
        let new = *self / rhs;
        self.x = new.x;
        self.y = new.y;
        self.z = new.z;
    }
}