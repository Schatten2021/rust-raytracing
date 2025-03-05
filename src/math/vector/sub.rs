use crate::math::general::Scalar;
use crate::math::vector::Vector3;
use std::ops::{Sub, SubAssign};

impl<T: Scalar> Sub<T> for &Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Vector3::new(
            self.x - rhs,
            self.y - rhs,
            self.z - rhs,
        )
    }
}
impl Sub<Vector3> for &Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: Vector3) -> Self::Output {
        Vector3::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}
impl Sub<&Vector3> for &Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: &Vector3) -> Self::Output {
        Vector3::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}
impl<T> SubAssign<T> for Vector3 where Vector3: Sub<T, Output = Vector3> {
    fn sub_assign(&mut self, scalar: T) {
        self.update((*self) - scalar);
    }
}
impl<T> Sub<T> for Vector3 where for<'a> &'a Vector3: Sub<T, Output = Vector3> {
    type Output = Self;
    fn sub(self, scalar: T) -> Self {
        (&self) - scalar
    }
}