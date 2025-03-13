pub mod sphere;
pub mod plane;
pub mod triangle;

use crate::math::Vector3;
use std::sync::{Arc, Mutex};

/// An object that can be raytraced/raymarched
#[derive(Clone)]
pub struct Object {
    /// The shape of the Object
    shape: Arc<Mutex<dyn CustomShape + Send + Sync>>,
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
    pub fn new<T: CustomShape + Send + Sync + 'static>(shape: T, material: Material) -> Self {
        let shape = Arc::new(Mutex::new(shape));
        Self { shape, material }
    }
    /// Returns the normal at the given position.
    /// Under the hood this is a call to [CustomShape::normal].
    ///
    /// # Arguments
    ///
    /// * `world_pos`: The position in world-space where the normal is requested from.
    ///
    /// returns: Vector3
    pub fn normal_at(&self, world_pos: Vector3) -> Vector3 {
        self.shape.lock().unwrap().normal(world_pos).norm()
    }
    /// Calculates the distance to the hit point.
    /// This is just a call to [CustomShape::distance] under the hood
    ///
    /// # Arguments
    ///
    /// * `ray_position`: The position of the ray in world space.
    /// * `ray_direction`: The direction of the ray in world space.
    ///
    /// returns: Option<f64>
    pub fn distance(&self, ray_position: Vector3, ray_direction: Vector3) -> Option<f64> {
        self.shape.lock().unwrap().distance(ray_position, ray_direction)
    }
}
pub trait CustomShape {
    /// Calculates the distance to the Object/Shape for a given ray.
    ///
    /// # Arguments
    ///
    /// * `ray_position`: The position of the ray in world-space.
    /// * `ray_direction`: The normalized direction of the ray in world-space.
    ///
    /// returns: Option<f64>
    ///
    /// # Notes
    /// * This version of the renderer only supports raytracing.
    /// This means, that the distance returned by this function is expected to be the distance to the hit point.
    /// If the object is not hit, this function should return [None].
    fn distance(&self, ray_position: Vector3, ray_direction: Vector3) -> Option<f64>;
    /// Calculates the normal vector of the Object/Shape at the given point.
    ///
    /// # Arguments
    ///
    /// * `world_position`: The position for which the normal is requested in world space.
    ///
    /// returns: Vector3
    fn normal(&self, world_position: Vector3) -> Vector3;
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