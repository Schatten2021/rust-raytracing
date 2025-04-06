use crate::math::Vector3;
use crate::object::CustomShape;
#[cfg(feature = "gpu")]
use crate::raytracing::gpu::GpuSerialize;
#[cfg(feature = "gpu")]
use crate::raytracing::gpu::object::GpuShape;

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
#[cfg(feature = "gpu")]
impl GpuSerialize for Plane {
    fn serialize(&self) -> Vec<u8> {
        self.position.serialize().into_iter()
            .chain([0; 4])
            .chain(self.normal.serialize())
            .chain([0; 4])
            .collect::<Vec<u8>>()
    }
}
#[cfg(feature = "gpu")]
impl GpuShape for Plane {
    fn struct_fields(&self) -> Vec<(String, String)> {
        vec![
            ("position".to_string(), "vec3<f32>".to_string()),
            ("normal".to_string(), "vec3<f32>".to_string()),
        ]
    }
    fn distance_code(&self) -> String {
        // let offset = ray_pos - self.position;
        // let norm = self.normal.norm();
        // let dir = ray_dir.norm();
        // // either the ray is going the opposite direction or it is coming from behind
        // if dir.dot(self.normal) >= 0. || offset.dot(self.normal) <= 0. {
        //     return None;
        // }
        // let t = offset.dot(norm) / dir.dot(norm);
        // let intersection_point = offset + dir * t;
        // Some((offset - intersection_point).len())
        "let offset: vec3<f32> = ray_position - current.position;
        let norm = normalize(current.normal);
        let dir = normalize(ray_direction);
        if dot(dir, norm) >= 0.0 || dot(offset, norm) <= 0.0 {
            return DistanceInfo(false, 0.0);
        }
        let t = dot(offset, norm) / dot(norm, dir);
        let intersection_point = offset + dir * t;
        return DistanceInfo(true, length(offset - intersection_point));
        ".to_string()
    }
    fn normal_calculation_code(&self) -> String {
        "return current.normal;".to_string()
    }
    fn object_type(&self) -> String {
        format!("{}::plane", module_path!())
    }
}