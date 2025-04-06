use crate::math::Vector3;
use crate::object::CustomShape;
#[cfg(feature = "gpu")]
use crate::raytracing::gpu::GpuSerialize;
#[cfg(feature = "gpu")]
use crate::raytracing::gpu::object::GpuShape;

#[derive(Clone, Debug)]
pub struct Sphere {
    pub position: Vector3,
    pub radius: f64,
}
impl Sphere {
    pub fn new(position: Vector3, radius: f64) -> Sphere {
        Self { position, radius }
    }
}
impl CustomShape for Sphere {
    fn distance(&self, ray_position: Vector3, ray_direction: Vector3) -> Option<f64> {
        let offset = ray_position - self.position;
        let ray_direction = ray_direction.norm();
        let a = ray_direction.dot(ray_direction);
        let b = 2f64 * offset.dot(ray_direction);
        let c = offset.dot(offset) - self.radius * self.radius;
        let discriminant = b * b - 4f64 * a * c;
        if discriminant <= 1e-100 {
            return None;
        }
        Some((-b - discriminant.sqrt()) / (2f64 * a))
    }
    fn normal(&self, world_position: Vector3) -> Vector3 {
        (world_position - self.position).norm()
    }
}
#[cfg(feature = "gpu")]
impl GpuSerialize for Sphere {
    fn serialize(&self) -> Vec<u8> {
        self.position.serialize().into_iter()
            .chain(self.radius.serialize())
            .collect()
    }
}
#[cfg(feature = "gpu")]
impl GpuShape for Sphere {
    fn struct_fields(&self) -> Vec<(String, String)> {
        vec![
            ("position".to_string(), "vec3<f32>".to_string()),
            ("radius".to_string(), "f32".to_string()),
        ]
    }

    fn distance_code(&self) -> String {
        // let offset = ray_position - self.position;
        // let ray_direction = ray_direction.norm();
        // let a = ray_direction.dot(ray_direction);
        // let b = 2f64 * offset.dot(ray_direction);
        // let c = offset.dot(offset) - self.radius * self.radius;
        // let discriminant = b * b - 4f64 * a * c;
        // if discriminant <= 1e-100 {
        //     return None;
        // }
        // Some((-b - discriminant.sqrt()) / (2f64 * a))
        "let offset: vec3<f32> = ray_position - current.position;
let ray_dir: vec3<f32> = normalize(ray_direction);
let a: f32 = dot(ray_dir, ray_dir);
let b: f32 = 2.0 * dot(offset, ray_dir);
let c = dot(offset, offset) - current.radius * current.radius;
let discriminant: f32 = b * b - 4.0 * a * c;
if (discriminant < 1e-100) {
    return DistanceInfo(false, 0.0);
} else {
    return DistanceInfo(true, (-b - sqrt(discriminant)) / (2.0 * a));
}".to_string()
    }

    fn normal_calculation_code(&self) -> String {
        "return normalize(world_position - current.position);".to_string()
    }
    fn object_type(&self) -> String {
        format!("{}::sphere", module_path!())
    }
    fn bounding_box_code(&self) -> String {
        "let min_x: vec3<f32> = current.position - current.radius;
let max_x: vec3<f32> = current.position + current.radius;
return BoundingBox(true, min_x, max_x);".to_string()
    }
}