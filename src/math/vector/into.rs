use crate::math::Vector3;
use crate::raytracing::gpu::GpuSerialize;

impl<A: Into<f64>, B: Into<f64>, C: Into<f64>> From<(A, B, C)> for Vector3 {
    fn from(tuple: (A, B, C)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}
impl<A: From<f64>, B: From<f64>, C: From<f64>> From<Vector3> for (A, B, C) {
    fn from(vec: Vector3) -> Self {
        (From::from(vec.x),
         From::from(vec.y),
         From::from(vec.z))
    }
}
impl<T: Into<f64> + Clone> From<[T; 3]> for Vector3 where {
    fn from(array: [T; 3]) -> Self {
        Self::new(Into::<f64>::into(array[0].clone()), Into::<f64>::into(array[1].clone()), Into::<f64>::into(array[2].clone()))
    }
}
impl<T: From<f64>> Into<[T; 3]> for Vector3 {
    fn into(self) -> [T; 3] {
        [T::from(self.x), T::from(self.y), T::from(self.z)]
    }
}
#[cfg(feature = "gpu")]
impl GpuSerialize for Vector3 {
    fn serialize(&self) -> Vec<u8> {
        self.x.serialize()
            .into_iter()
            .chain(self.y.serialize())
            .chain(self.z.serialize())
            .collect()
    }
}