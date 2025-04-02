use crate::math::Vector3;
use std::sync::{Arc, Mutex};
use crate::raytracing::gpu::GpuSerialize;

/// An object that can be raytraced/raymarched
#[derive(Clone)]
pub struct Object {
    /// The shape of the Object
    pub(crate) shape: Arc<Mutex<dyn GpuShape + Send + Sync>>,
    /// The material of the Object
    pub material: Material,
}
impl Object {
    /// Creates a new object
    ///
    /// # Arguments
    ///
    /// * `shape`: The shape of the new Object.
    /// * `material`: The material that the object has.
    ///
    /// returns: Object
    pub fn new<T: GpuShape + Send + Sync + 'static>(shape: T, material: Material) -> Self {
        let shape = Arc::new(Mutex::new(shape));
        Self { shape, material }
    }
    pub(crate) fn normal_code(&self) -> String {
        self.shape.lock().unwrap().normal_calculation_code()
    }
    pub(crate) fn distance(&self) -> String {
        self.shape.lock().unwrap().distance_code()
    }
    pub(crate) fn struct_code(&self) -> Vec<(String, String)> {
        self.shape.lock().unwrap().struct_fields()
    }
    // pub(crate) fn to_gpu_object(&self, object_id: u64) -> GPUSendableObject {
    //     GPUSendableObject {
    //         base_color: self.material.base_color.into(),
    //         emission: self.material.emission_color.into(),
    //         roughness: self.material.roughness.into(),
    //         object_id,
    //     }
    // }
    pub(crate) fn gpu_serialize(&self, object_id: u32) -> Vec<u8> {
        self.material.base_color.serialize().into_iter()
            .chain(self.material.roughness.serialize())
            .chain(self.material.emission_color.serialize())
            .chain(object_id.to_le_bytes())
            .collect::<Vec<_>>()
    }
}
pub trait GpuShape: GpuSerialize {
    /// generates the fields and names for the struct of this shape
    ///
    /// # returns
    /// Vec<(field name, field type)>
    fn struct_fields(&self) -> Vec<(String, String)>;
    /// generates the wgsl code for generating the distance.
    ///
    /// function takes two `vec3<f32>`s. 1st one is the position, 2nd one is the direction in world space.
    /// The function should return a DistanceInfo object which is constructed via 1 bool and 1 f32.
    /// The boolean dictates, whether the object was actually hit or not and the f32 dictates the distance.
    /// If the Object isn't hit (boolean  is false) the distance is ignored.
    ///
    /// Example code (sphere):
    /// ``` wgsl
    /// // define the struct for code completion/etc.
    /// struct Sphere {
    ///     position: vec3<f32>,
    ///     radius: f32,
    /// }
    /// // redefine the value for easier use (code completion/etc.)
    /// Sphere self = Sphere(self.position, self.radius);
    /// // define an inner function (again, easier use)
    /// fn distance(self: Sphere, ray_position: vec3<f32>, ray_direction: vec3<f32>) -> DistanceInfo {
    ///     ...
    /// }
    /// // call the function.
    /// return distance(self, ray_position, ray_direction);
    /// ```
    fn distance_code(&self) -> String;
    /// returns wgsl code for calculating the normal at a given point.
    ///
    /// The function takes one `vec3<f32>`: The position at which the normal is requested.
    /// The function should return one `vec<f32>`: The normal at the given point.
    ///
    /// ``` wgsl
    /// fn normal(position: vec3<f32>) -> vec3<f32> {
    ///     ...
    /// }
    /// return normal(position);
    /// ```
    fn normal_calculation_code(&self) -> String;
    // fn object_type(&self) -> String;
}
/// represents the material of an [Object]
#[derive(Clone, Debug)]
pub struct Material {
    /// The base color of the object. This will be, through the process of Pathtracing, slightly mixed with surrounding colors.
    pub base_color: Vector3,
    /// The color of emissions on the object.
    pub emission_color: Vector3,
    /// How rough the material is.
    /// 0 means rays are only reflected, 1 means rays bounce randomly.
    ///
    /// The lower the number, the more the rays bounce towards a full reflection.
    pub roughness: f64,
}
impl Material  {
    /// creates a new material with the given specs
    pub const fn new(base_color: Vector3, emission_color: Vector3, roughness: f64) -> Self {
        Self { base_color, emission_color, roughness }
    }
    /// Creates a new material with only a color component.
    ///
    /// # Arguments
    ///
    /// * `color`: The target color of the material
    ///
    /// returns: Material
    ///
    /// # Examples
    ///
    /// ```
    /// use rtx::math::Vector3;
    /// use rtx::object::Material;
    /// let yellow = Material::colored(Vector3::new(1, 1, 0));
    /// let red = Material::colored(Vector3::x());
    /// ```
    pub const fn colored(color: Vector3) -> Self {
        Self::new(color, Vector3::zeros(), 1f64)
    }
    /// Creates a new material with only an emissions component.
    ///
    /// # Arguments
    ///
    /// * `light_color`: The color of the light that the object emits.
    ///
    /// returns: Material
    ///
    /// # Examples
    ///
    /// ```
    /// use rtx::math::Vector3;
    /// use rtx::object::Material;
    /// let light = Material::light(Vector3::ones());
    /// let sun = Material::light(Vector3::new(1, 0.8, 0.5)); // orange-ish light
    /// ```
    pub const fn light(light_color: Vector3) -> Self {
        Self::new(Vector3::zeros(), light_color, 1f64)
    }
    pub const fn mirror() -> Self {
        Self::new(Vector3::ones(), Vector3::zeros(), 1f64)
    }
}

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub(super) struct GPUSendableObject {
//     pub base_color: Vector3,
//     pub emission: Vector3,
//     pub roughness: f64,
//     pub object_id: u64,
// }