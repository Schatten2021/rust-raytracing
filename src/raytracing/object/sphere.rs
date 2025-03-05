use crate::math::Vector3;
use crate::object::Object;

#[derive(Clone, Debug)]
pub struct Sphere {
    pub position: Vector3,
    pub radius: f64,
    pub col: Vector3,
    pub emissions: Vector3,
}
impl Sphere {
    pub fn new(position: Vector3, radius: f64, col: Vector3, emissions: Vector3) -> Sphere {
        Self { position, radius, col, emissions, }
    }
}
impl Object for Sphere {
    fn distance(&self, pos: Vector3, dir: Vector3) -> Option<f64> {
        let offset = pos - self.position;
        let a = dir.dot(dir);
        let b = 2f64 * offset.dot(dir);
        let c = offset.dot(offset) - self.radius * self.radius;
        let discriminant = b * b - 4f64 * a * c;
        if discriminant <= 1e-100 {
            return None;
        }
        Some((-b - discriminant.sqrt()) / (2f64 * a))
    }
    fn get_base(&self, _world_hit_pos: Vector3) -> Vector3 {
        self.col
    }
    fn get_emission(&self, _world_hit_pos: Vector3) -> Vector3 {
        self.emissions
    }
    fn normal(&self, world_position: Vector3) -> Vector3 {
        (world_position - self.position).norm()
    }
}