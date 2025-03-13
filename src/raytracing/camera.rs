use crate::math::{Mat3x3, Vector3};

#[derive(Debug, Clone)]
/// represents a camera in the scene
pub struct Camera {
    /// The _horizontal_ fov of the camera in radians.
    pub fov: f64,
    /// The position of the camera in the world
    pub position: Vector3,
    direction: Vector3,
    to_cam_space: Mat3x3,
    to_world_space: Mat3x3,
}

impl Camera {
    /// creates a new camera to be used for renders
    pub fn new(position: Vector3, direction: Vector3, fov: f64) -> Self {
        let to_world_space = Self::derive_to_world_space_mat(direction);
        Camera {
            fov,
            position,
            direction,
            to_cam_space: to_world_space.inverse(),
            to_world_space,
        }
    }
    /// returns the direction the camera is facing.
    pub fn get_direction(&self) -> Vector3 {
        self.direction
    }
    /// sets the direction the camera is facing.
    /// This has to be a separate function, because the matrices have to be updated.
    pub fn set_direction(&mut self, direction: Vector3) {
        let to_world_space = Self::derive_to_world_space_mat(self.direction);
        self.to_cam_space = to_world_space.inverse();
        self.to_world_space = to_world_space;
        self.direction = direction;
    }
    /// derives the matrix from camera space to world space
    fn derive_to_world_space_mat(direction: Vector3) -> Mat3x3 {
        let cam_forward = direction.norm();
        let cam_right = cam_forward.cross(Vector3::new(0.,0.,-1.));
        let cam_up = cam_forward.cross(cam_right);
        Mat3x3::new(cam_right,
                    cam_up,
                    cam_forward).transpose()
    }
    /// Turns a vector from world space to cam space
    pub fn to_cam_space(&self, vec: Vector3) -> Vector3 {
        self.to_cam_space * (vec - self.position)
    }
    /// Moves a vector from cam space to world space
    pub fn to_world_space(&self, vec: Vector3) -> Vector3 {
        self.to_world_space * vec + self.position
    }
    /// Rotates a vector into world space (aka assumes that the camera is positioned at (0|0|0)).
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector to be rotated
    ///
    /// returns: Vector3
    pub fn rotate_to_world_space(&self, vec: Vector3) -> Vector3 {
        self.to_world_space * vec
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_world_space() {
        let cam = Camera::new(Vector3::zeros(), Vector3::x(), 90f64.to_radians());
        assert_eq!(cam.to_cam_space(Vector3::x()), Vector3::z());
        assert_eq!(cam.to_cam_space(Vector3::y()), Vector3::x());
        assert_eq!(cam.to_cam_space(Vector3::z()), Vector3::y());
    }
    #[test]
    fn from_cam_space() {
        let cam = Camera::new(Vector3::zeros(), Vector3::x(), 90f64.to_radians());
        assert_eq!(cam.to_world_space(Vector3::x()), Vector3::y());
        assert_eq!(cam.to_world_space(Vector3::y()), Vector3::z());
        assert_eq!(cam.to_world_space(Vector3::z()), Vector3::x());
    }
    #[test]
    fn from_cam_space_2() {
        let cam = Camera::new(Vector3::zeros(), Vector3::y(), 90f64.to_radians());
        assert_eq!(cam.to_world_space(Vector3::x()), -Vector3::x());
        assert_eq!(cam.to_world_space(Vector3::y()), Vector3::z());
        assert_eq!(cam.to_world_space(Vector3::z()), Vector3::y());
    }
    #[test]
    fn from_world_space_2() {
        let cam = Camera::new(Vector3::zeros(), Vector3::y(), 90f64.to_radians());
        assert_eq!(cam.to_cam_space(Vector3::x()), -Vector3::x());
        assert_eq!(cam.to_cam_space(Vector3::y()), Vector3::z());
        assert_eq!(cam.to_cam_space(Vector3::z()), Vector3::y());
    }
}