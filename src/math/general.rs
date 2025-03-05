pub(super) trait Scalar {
    fn to_scalar(self) -> f64;
}
impl<T: Scalar + Clone> Scalar for &T {
    fn to_scalar(self) -> f64 {
        (*self).clone().to_scalar()
    }
}
macro_rules! impl_scalar {
    ($($t:ty),+) => {
        $(
            impl Scalar for $t {
                fn to_scalar(self) -> f64 {
                    self as f64
                }
            }
        )*
    }
}
impl_scalar!(f32, f64, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);