pub mod sphere;
pub mod plane;
pub mod triangle;

use std::f64::consts::PI;
use std::fmt::Debug;
use crate::math::Vector3;

/// An object that can be raytraced/raymarched
pub trait Object: Debug {
    /// Calculates the distance from the object given the position and direction of the ray.
    ///
    /// # Arguments
    ///
    /// * `pos`: The position of the ray
    /// * `dir`: The direction the ray is traveling in
    ///
    /// returns: Option<f64> Returns None, when the object won't be hit
    fn distance(&self, pos: Vector3, dir: Vector3) -> Option<f64>;
    /// Returns the base color of the Object.
    ///
    /// # Arguments
    ///
    /// * `world_hit_pos`: The position the object was hit in. Can be used to generate Textures.
    ///
    /// returns: Vector3 The resulting color
    fn get_base(&self, world_hit_pos: Vector3) -> Vector3;

    /// Returns the emission of the Object.
    ///
    /// # Arguments
    ///
    /// * `world_hit_pos`: The position the object was hit in, in world space.
    ///
    /// returns: Vector3 The Emission color
    fn get_emission(&self, world_hit_pos: Vector3) -> Vector3;

    /// Updates the direction the ray is traveling in. This can be a fully random direction, the Ray reflected, or something more complicated.
    ///
    /// # Arguments
    ///
    /// * `world_hit_pos`: The position the ray was hit in. Can be used for calculating normals/etc.
    /// * `ray_dir`: The direction the ray was traveling in. Useful for BRDFs.
    ///
    /// returns: Vector3 The new direction of the ray
    #[allow(unused_variables)]
    fn update_dir(&self, world_hit_pos: Vector3, ray_dir: Vector3) -> Vector3 {
        let normal = self.normal(world_hit_pos);
        let pitch = fastrand::f64() * PI;
        let jaw = fastrand::f64() * PI;
        let random_dir = Vector3::new(
            pitch.sin(),
            jaw.sin(),
            pitch.cos() * jaw.cos(),
        );
        if random_dir.dot(normal) > 0. {
            random_dir
        } else {
            -random_dir
        }
    }
    /// Returns the mode the object is in. This can either be RayTracing or RayMarching.
    /// For RayMarching the distance will be used as the minimum distance to the object, whereas with RayTracing the distance will be seen as the distance until hit.
    /// The default is RayTracing, as it is more performant (only having to calculate the distance to the object once instead of multiple times).
    fn mode(&self) -> ObjectMode {ObjectMode::RayTracing}

    /// Returns the normal of the object at the given position.
    fn normal(&self, world_position: Vector3) -> Vector3;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The mode an object operates in.
pub enum ObjectMode {
    /// Objects that can be raytraced.
    /// Objects that have this as their mode are expected to return the absolute distance to a hitpoint.
    RayTracing,
    /// Objects that are raymarched.
    /// Objects that have this as their mode are expected to return the minimum distance a Ray can travel where it doesn't hit the object.
    RayMarching,
}