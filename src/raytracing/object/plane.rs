use crate::math::Vector3;
use crate::object::CustomShape;

#[derive(Clone, Debug)]
pub struct Plane {
    pub position: Vector3,
    pub normal: Vector3,
}
impl Plane {
    pub fn new(position: Vector3, normal: Vector3) -> Plane {
        Self { position, normal }
    }
}
impl CustomShape for Plane {
    fn distance(&self, ray_pos: Vector3, ray_dir: Vector3) -> Option<f64> {
        let offset = ray_pos - self.position;
        let norm = self.normal.norm();
        let dir = ray_dir.norm();
        // either the ray is going the opposite direction or it is coming from behind
        if dir.dot(self.normal) >= 0. || offset.dot(self.normal) <= 0. {
            return None;
        }
        let t = offset.dot(norm) / dir.dot(norm);
        let intersection_point = offset + dir * t;
        Some((offset - intersection_point).len())
    }

    fn normal(&self, _relative_position: Vector3) -> Vector3 {
        self.normal
    }
}