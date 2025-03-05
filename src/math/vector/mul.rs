use std::ops::{Mul, MulAssign};
use crate::math::general::Scalar;
use crate::math::vector::Vector3;

impl<T> Mul<T> for Vector3 where for<'a> &'a Vector3: Mul<T, Output = Vector3> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self {
        (&self) * scalar
    }
}
impl<T: Scalar> Mul<T> for &Vector3 {
    type Output = Vector3;
    fn mul(self, scalar: T) -> Self::Output {
        let scalar = scalar.to_scalar();
        Vector3::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
        )
    }
}
impl Mul<Vector3> for &Vector3 {
    type Output = Vector3;
    fn mul(self, other: Vector3) -> Self::Output {
        Vector3::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
        )
    }
}
impl Mul<&Vector3> for &Vector3 {
    type Output = Vector3;
    fn mul(self, other: &Vector3) -> Self::Output {
        Vector3::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
        )
    }
}
impl<T> MulAssign<T> for Vector3 where Vector3: Mul<T, Output = Vector3> {
    fn mul_assign(&mut self, scalar: T) {
        self.update(*self * scalar);
    }
}
