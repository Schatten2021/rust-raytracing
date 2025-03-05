use crate::math::Vector3;
use crate::object::Object;
#[derive(Clone, Debug)]
pub struct Plane {
    pub position: Vector3,
    pub normal: Vector3,
    pub col: Vector3,
    pub emission: Vector3,
}
impl Plane {
    pub fn new(position: Vector3, normal: Vector3, col: Vector3, emission: Vector3) -> Plane {
        Self { position, normal, col, emission }
    }
}
impl Object for Plane {
    fn distance(&self, pos: Vector3, dir: Vector3) -> Option<f64> {
        let offset = pos - self.position;
        let norm = self.normal.norm();
        let dir = dir.norm();
        // either the ray is going the opposite direction or it is coming from behind
        if dir.dot(self.normal) >= 0. || offset.dot(self.normal) <= 0. {
            return None;
        }
        let t = offset.dot(norm) / dir.dot(norm);
        let intersection_point = offset + dir * t;
        Some((offset - intersection_point).len())
    }
    fn get_base(&self, _world_hit_pos: Vector3) -> Vector3 {
        self.col
    }
    fn get_emission(&self, _world_hit_pos: Vector3) -> Vector3 {
        self.emission
    }
    fn normal(&self, _world_position: Vector3) -> Vector3 {
        self.normal.norm()
    }
}