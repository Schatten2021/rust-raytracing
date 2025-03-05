pub mod math;
pub mod raytracing;
pub use raytracing::camera::Camera;
pub use raytracing::scene::{ Scene, Config };
pub use raytracing::object;


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
