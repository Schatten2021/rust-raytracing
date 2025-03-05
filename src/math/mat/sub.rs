use std::ops::{Sub, SubAssign};
use crate::math::general::Scalar;
use crate::math::mat::Mat3x3;

impl<T: Scalar> Sub<T> for &Mat3x3 {
    type Output = Mat3x3;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Mat3x3::new(
            self.x - rhs,
            self.y - rhs,
            self.z - rhs,
        )
    }
}
impl Sub<Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;
    fn sub(self, rhs: Mat3x3) -> Self::Output {
        Mat3x3::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}
impl Sub<&Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;
    fn sub(self, rhs: &Mat3x3) -> Self::Output {
        Mat3x3::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}
impl<T> Sub<T> for Mat3x3 where for<'a> &'a Mat3x3: Sub<T, Output = Mat3x3> {
    type Output = Mat3x3;
    fn sub(self, rhs: T) -> Self::Output {
        (&self) - rhs
    }
}
impl<T> SubAssign<T> for Mat3x3 where Mat3x3: Sub<T, Output = Mat3x3> {
    fn sub_assign(&mut self, rhs: T) {
        let res = *self - rhs;
        self.x = res.x;
        self.y = res.y;
        self.z = res.z;
    }
}