use crate::math::general::Scalar;
use crate::math::mat::Mat3x3;
use std::ops::{Mul, MulAssign};
use crate::math::vector::Vector3;

impl<I, O> Mul<I> for Mat3x3 where for<'a> &'a Mat3x3: Mul<I, Output = O> {
    type Output = O;
    fn mul(self, rhs: I) -> Self::Output {
        (&self) * rhs
    }
}
impl<T: Scalar> Mul<T> for &Mat3x3 {
    type Output = Mat3x3;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Mat3x3::new(
            self.x * rhs,
            self.y * rhs,
            self.z * rhs,
        )
    }
}
impl Mul<&Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;
    fn mul(self, rhs: &Mat3x3) -> Self::Output {
        Mat3x3::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
        )
    }
}

impl<T> MulAssign<T> for Mat3x3 where Mat3x3: Mul<T, Output = Mat3x3> {
    fn mul_assign(&mut self, rhs: T) {
        let new = *self * rhs;
        self.x = new.x;
        self.y = new.y;
        self.z = new.z;
    }
}
impl Mul<&Vector3> for &Mat3x3 {
    type Output = Vector3;
    fn mul(self, rhs: &Vector3) -> Self::Output {
        Vector3::new(
            rhs.dot(self.x),
            rhs.dot(self.y),
            rhs.dot(self.z),
        )
    }
}
impl Mul<Vector3> for &Mat3x3 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Self::Output {
        self * &rhs
    }
}