use crate::math::general::Scalar;
use crate::math::vector::Vector3;
use std::ops::{Div, DivAssign};

impl<T> Div<T> for Vector3 where for<'a> &'a Vector3: Div<T, Output = Vector3> {
    type Output = Self;
    fn div(self, other: T) -> Self {
        (&self) / other
    }
}
impl<T: Scalar> Div<T> for &Vector3 {
    type Output = Vector3;
    fn div(self, other: T) -> Self::Output {
        let other = other.to_scalar();
        Vector3::new(
            self.x / other,
            self.y / other,
            self.z / other,
        )
    }
}
impl Div<Vector3> for &Vector3 {
    type Output = Vector3;
    fn div(self, other: Vector3) -> Self::Output {
        Vector3::new(
            self.x / other.x,
            self.y / other.y,
            self.z / other.z,
        )
    }
}
impl Div<&Vector3> for Vector3 {
    type Output = Vector3;
    fn div(self, other: &Vector3) -> Self::Output {
        Vector3::new(
            self.x / other.x,
            self.y / other.y,
            self.z / other.z,
        )
    }
}
impl<T> DivAssign<T> for Vector3 where Vector3: Div<T, Output = Vector3> {
    fn div_assign(&mut self, other: T) {
        let res = *self / other;
        self.update(res)
    }
}