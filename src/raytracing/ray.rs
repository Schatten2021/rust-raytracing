use crate::math::Vector3;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Ray {
    pub position: Vector3,
    pub direction: Vector3,
    /// The final color of the ray
    pub resulting_color: Vector3,
    /// The color of the light ray, if the ray were to hit something
    pub light_color: Vector3,
}

impl Ray {
    pub(crate) fn new(position: Vector3, direction: Vector3) -> Self {
        Self {
            position,
            direction,
            resulting_color: Vector3::zeros(),
            light_color: Vector3::ones(),
        }
    }
}