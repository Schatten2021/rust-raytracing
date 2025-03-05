use crate::math::general::Scalar;
use crate::math::vector::Vector3;
use std::ops::{Add, AddAssign};

impl<T: Scalar> Add<T> for &Vector3 {
    type Output = Vector3;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.to_scalar();
        Vector3::new(
            self.x + rhs,
            self.y + rhs,
            self.z + rhs,
        )
    }
}
impl Add<Vector3> for &Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Self::Output {
        Vector3::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}
impl<T> Add<T> for Vector3 where for<'a> &'a Vector3: Add<T, Output = Vector3> {
    type Output = Vector3;
    fn add(self, rhs: T) -> Self::Output {
        &self + rhs
    }
}
impl<T> AddAssign<T> for Vector3 where Vector3: Add<T, Output = Vector3> {
    fn add_assign(&mut self, rhs: T) {
        let res = *self + rhs;
        self.x = res.x;
        self.y = res.y;
        self.z = res.z;
    }
}
// TODO: implement the reverse