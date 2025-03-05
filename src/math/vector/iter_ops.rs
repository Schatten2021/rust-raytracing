use std::iter::{Product, Sum};
use crate::math::Vector3;

impl Sum for Vector3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vector3::zeros(), |a, b| a + b)
    }
}
impl Product for Vector3 {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vector3::ones(), |a, b| a * b)
    }
}