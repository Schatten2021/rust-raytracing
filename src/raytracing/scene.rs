use crate::math::Vector3;
use crate::raytracing::camera::Camera;
use crate::raytracing::object::Object;
use crate::raytracing::ray::Ray;
use std::thread;

#[cfg(feature = "images")]
use image::{
    ImageBuffer,
    Rgb
};
#[derive(Clone, Debug)]
pub struct Config {
    // /// determines, at which distance a ray is seen as having hit an object
    // pub raymarch_collision_threshold: f64,
    // /// Determines, how much of the minimum distance is gone per step.
    // /// This is done so that the ray won't go directly into the closest object.
    // ///
    // /// Should be made as close to 1 as possible but not 1.
    // ///
    // /// In case of visual artifacts, where a ray goes through an object, make this smaller.
    // ///
    // /// **DO NOT make this larger than 1.**
    // pub raymarch_step_size_multiplier: f64,
    /// determines, how many rays are shot out per pixel. The more, the lower quality, the higher, the more costly the renderer will get.
    pub rays_per_pixel: usize,
    /// The maximum number of bounces a ray can make.
    /// The higher, the more indirect lighting will appear, the less, the faster the rendering will be.
    pub max_bounces: usize,
    /// The maximum amount of steps a ray can make.
    /// The higher, the more accurate the image, the lower, the faster the image renders.
    pub max_steps: usize,
    /// The distance at which point the renderer assumes that the ray won't hit anything.
    pub max_distance: f64,
    /// The distance of the focal point.
    pub focal_length: f64,
    /// The maximum offset of each ray at the focal point. Can be used for a bit of anti-Aliasing at the focal point
    pub focal_offset: f64,
    /// The maximum offset of each ray's start position. This makes the focus effect stronger/weaker.
    pub non_focal_offset: f64,
}
macro_rules! reassign {
    ($self:ident, $field:ident) => {
        {
            let mut new = $self.clone();
            new.$field = $field;
            new
        }
    }
}
impl Config {
    // pub fn with_raymarch_collision_threshold(&self, raymarch_collision_threshold: f64) -> Self {
    //     reassign!(self, raymarch_collision_threshold)
    // }
    // pub fn with_raymarch_step_size_multiplier(&self, raymarch_step_size_multiplier: f64) -> Self {
    //     reassign!(self, raymarch_step_size_multiplier)
    // }
    pub fn with_rays_per_pixel(&self, rays_per_pixel: usize) -> Self {
        reassign!(self, rays_per_pixel)
    }
    pub fn with_max_bounces(&self, max_bounces: usize) -> Self {
        reassign!(self, max_bounces)
    }
    pub fn with_max_steps(&self, max_steps: usize) -> Self {
        reassign!(self, max_steps)
    }
    pub fn with_max_distance(&self, max_distance: f64) -> Self {
        reassign!(self, max_distance)
    }
    pub fn with_focal_length(&self, focal_length: f64) -> Self {
        reassign!(self, focal_length)
    }
    pub fn with_focal_offset(&self, focal_offset: f64) -> Self {
        reassign!(self, focal_offset)
    }
    pub fn with_non_focal_offset(&self, non_focal_offset: f64) -> Self {
        reassign!(self, non_focal_offset)
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            // raymarch_collision_threshold: 1e-6,
            // raymarch_step_size_multiplier: 0.99,
            rays_per_pixel: 16,
            max_bounces: 10,
            max_steps: 100,
            max_distance: 1e12,
            focal_length: 10f64,
            focal_offset: 1e-4,
            non_focal_offset: 1e-1,
        }
    }
}
#[derive(Clone)]
pub struct Scene {
    pub objects: Vec<Object>,
    /// The camera that this scene is rendered from
    pub camera: Camera,
    /// The configuration of this scene.
    pub config: Config,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            config: Config::default(),
            camera: Camera::new((0, 0, 0).into(), (1, 0, 0).into(), 90f64),
            objects: Vec::new(),
        }
    }
}
impl Scene {
    /// Creates a new Scene.
    ///
    /// # Arguments
    ///
    /// * `config`: The configuration for the scene
    /// * `camera`: The camera to be used for rendering.
    ///
    /// returns: Scene
    ///
    /// # Examples
    ///
    /// ```
    /// use rtx::{Camera, Scene, Config};
    ///
    /// let scene = Scene::new(Config::default().with_rays_per_pixel(32), Camera::new((-1,0,0).into(), (1,0,0).into(), 90f64));
    /// ```
    pub fn new(config: Config, camera: Camera) -> Self {
        Self {
            config,
            camera,
            objects: Vec::new(),
        }
    }
    /// Adds a new Object to the scene.
    ///
    /// # Arguments
    ///
    /// * `object`: The object to add
    ///
    /// returns: ()
    pub fn add_object(&mut self, object: Object) -> () {
        self.objects.push(object);
    }
    /// Renders the scene as an image.
    ///
    /// # Arguments
    ///
    /// * `width`: The width of the resulting image in pixels
    /// * `height`: The height of the resulting image in pixels
    ///
    /// returns: Vec<Vec<Vector3, Global>, Global>
    ///     The image, indexed with `img[y][x]`
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn render(&self, width: usize, height: usize) -> Vec<Vec<Vector3>> {
        let vertical_fov = (height as f64) / (width as f64) * self.camera.fov;
        vec![vec![Vector3::default(); width]; height]
            .into_iter()
            .enumerate()
            .map(|(y, row)| {
                let this = self.clone();
                thread::spawn(move || {
                    // let row_index = y;
                    let y = (y as f64) / (height as f64);
                    let res = row.into_iter()
                        .enumerate()
                        .map(|(x, _pixel)| {
                            let x = (x as f64) / (width as f64);
                            let uv = (x, y);
                            this.render_pixel(uv, vertical_fov)
                        })
                        .collect::<Vec<_>>();
                    // let done = (y) / (height as f64);
                    // println!("rendered row {} out of {} ({:.2}%)", row_index, height, done * 100.0);
                    res
                })
            })
            .collect::<Vec<_>>().into_iter()
            .map(|t| t.join().unwrap())
            .collect::<Vec<_>>()
    }
    /// Renders the scene to an ImageBuffer. Requires the `images` feature
    #[cfg(feature = "images")]
    pub fn render_to_image(&self, width: usize, height: usize) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let img = self.render(width, height);
        ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            let col = img[height - y as usize - 1][x as usize].clone() * 256;
            Rgb([col.x as u8, col.y as u8, col.z as u8])
        })
        // let vertical_fov = (height as f64) / (width as f64) * self.camera.fov;
        // ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        //     if x == 0 {
        //         let done = (y as f64) / (height as f64);
        //         println!("rendering row {} out of {} ({:.2}%)", y, height, done * 100.0);
        //     }
        //     let x = (x as f64) / (width as f64);
        //     let y = (y as f64) / (height as f64);
        //     let col = self.render_pixel((x, 1.0 - y), vertical_fov) * 256;
        //     Rgb([col.x as u8, col.y as u8, col.z as u8])
        // })
    }

}
impl Scene {
    fn render_pixel(&self, uv: (f64, f64), vertical_fov: f64) -> Vector3 {
        let ray_dir = self.get_ray_dir(uv.0, uv.1, vertical_fov);
        let ray = Ray::new(self.camera.position, ray_dir);
        avg(
            (0..self.config.rays_per_pixel)
                .map(|_| {
                    // focal point calculations
                    let mut ray = ray.clone();
                    let ray_position = ray.position + Vector3::random() * self.config.non_focal_offset;
                    let focal_point = ray.position + ray.direction * self.config.focal_length;
                    let target_point = focal_point + Vector3::random() * self.config.focal_offset;
                    let ray_direction = target_point - ray_position;
                    ray.position = ray_position;
                    ray.direction = ray_direction.norm();
                    // actual rendering happening
                    self.render_ray(ray)
                })
        )
    }
    fn get_ray_dir(&self, x: f64, y: f64, vertical_fov: f64) -> Vector3 {
        let angle_x = self.camera.fov * (x - 0.5);
        let angle_y = vertical_fov * (y - 0.5);
        let cam_space_dir = Vector3::new(
            angle_x.sin(),
            angle_y.sin(),
            angle_x.cos() * angle_y.cos()
        );
        self.camera.rotate_to_world_space(cam_space_dir)
    }
    fn render_ray(&self, mut ray: Ray) -> Vector3 {
        if self.objects.is_empty() {
            return ray.resulting_color;
        }
        for _ in 0..(self.config.max_bounces+1) {
            if ray.light_color == Vector3::zeros() {
                break;
            }
            let rtx_hit = self.closest_object(ray);
            match rtx_hit {
                Some((dst, obj)) => {
                    ray.position += ray.direction * dst;
                    ray_hit(&mut ray, obj);
                    // return ray.direction;
                },
                None => break,
            }
        }
        ray.resulting_color
    }
    fn closest_object(&self, ray: Ray) -> Option<(f64, &Object)> {
        self.objects.iter()
            .filter_map(|obj| {
                obj.distance(ray.position, ray.direction)
                    .map(|dst| (dst, obj))
            })
            .filter(|(dst, _)| dst.is_normal() && dst.is_sign_positive())
            .min_by(move |(dst, _), (dst2, _)| dst.total_cmp(dst2))
    }
}
fn avg<I: ExactSizeIterator, R>(iter: I) -> R
where <I as Iterator>::Item: std::iter::Sum,
    <I as Iterator>::Item: std::ops::Div<usize, Output = R>
{
    let length = iter.len();
    iter.sum::<I::Item>() / length
}
fn ray_hit(ray: &mut Ray, object: &Object) {
    // let surface_normal = object.normal_at(ray.position);
    // let random_dir = Vector3::random_direction();
    // let reflected_dir = ray.direction - surface_normal * (ray.direction.dot(surface_normal)) * 2;
    // let offset = random_dir - reflected_dir;
    //
    // let reflection_index = (1.0 - object.material.roughness);
    // let new_dir = random_dir + offset * reflection_index;
    // let random_dir = Vector3::random_direction();
    // let random_dir = if random_dir.dot(surface_normal) < 0.0 {
    //     -random_dir
    // } else {random_dir};
    // let reflected_dir = ray.direction - surface_normal * (ray.direction.dot(surface_normal)) * 2;
    // let new_dir = if fastrand::f64() > object.material.roughness * ( random_dir.dot(reflected_dir) * 0.5 + 0.5) { reflected_dir } else {random_dir};

    ray.direction = random_bounce_dir(ray.direction, object.normal_at(ray.position), object.material.roughness);
    ray.resulting_color += ray.light_color * object.material.emission_color;
    ray.light_color *= object.material.base_color;
}
fn random_bounce_dir(ray_dir: Vector3, surface_normal: Vector3, surface_roughness: f64) -> Vector3 {
    let random_dir = Vector3::random_direction();
    let reflection_dir = ray_dir - surface_normal * 2 * ray_dir.dot(surface_normal);
    let random_to_reflection_dir = reflection_dir - random_dir;
    let reflection_mult = 1.0 - surface_roughness;
    let final_direction = random_dir + random_to_reflection_dir * reflection_mult;

    let final_direction = final_direction.norm();
    if final_direction.dot(surface_normal) > 0.0 {
        final_direction
    } else {
        -final_direction
    }
}