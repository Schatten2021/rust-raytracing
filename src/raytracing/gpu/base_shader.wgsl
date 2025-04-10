const PI = acos(0.0) * 2.0;
struct Camera {
    pos: vec3<f32>,
    dir: vec3<f32>,
    fov: f32,
}
struct Object {
    base_color: vec3<f32>,
    roughness: f32, // placed here to make alignement easier
    emission_color: vec3<f32>,
    object_id: u32, // placed here for easier alignement
    object_index: u32,
    //vec3<f32> requires a 16 bit alignement, that's why those above are where they are.
}
struct BoundingBox {
    has_box: bool,
    min: vec3<f32>,
    max: vec3<f32>,
}
struct Config {
    rays_per_pixel: u32,
    max_bounces: u32,
    focal_length: f32,
    focal_offset: f32,
    non_focal_offset: f32,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> aspect_ratio: f32;
@group(0)
@binding(2)
var<uniform> config: Config;

@group(0)
@binding(3)
var<storage, read> objects: array<Object>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Define three positions that cover the whole screen.
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // bottom-left
        vec2<f32>( 3.0, -1.0), // bottom-right (overshoots)
        vec2<f32>(-1.0,  3.0)  // top-left (overshoots)
    );
    var position: vec2<f32> = positions[vertex_index];
    var output: VertexOutput;
    output.clip_position = vec4<f32>(position, 0.0, 1.0);
    // Map clip space [-1, 1] to UV space [0, 1]:
    output.uv = position * vec2<f32>(0.5, -0.5); // + vec2<f32>(0.5, 0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    initRngSeed(input.uv);
    let angle = input.uv * vec2<f32>(camera.fov, camera.fov / aspect_ratio);
    let cam_space_dir: vec3<f32> = vec3<f32>(sin(angle), cos(angle.x) * cos(angle.y));

    let cam_space_forward: vec3<f32> = normalize(camera.dir);
    let cam_space_right: vec3<f32> = cross(cam_space_forward, vec3<f32>(0.0, 0.0, 1.0));
    let cam_space_up: vec3<f32> = cross(cam_space_forward, cam_space_right);
    let to_cam_space_mat: mat3x3<f32> = mat3x3<f32>(cam_space_right, cam_space_up, cam_space_forward);

    let ray_dir = inverse3x3(to_cam_space_mat) * cam_space_dir;

    let ray = Ray(camera.pos, ray_dir, vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.0, 0.0, 0.0));
    var color: vec3<f32> = vec3(0.0, 0.0, 0.0);
    let target_point = ray.position + ray.direction * config.focal_length;
    for (var i: u32 = 0u; i < config.rays_per_pixel; i++) {
        var current_ray = ray;
        let ray_target = target_point + random_direction() * config.focal_offset;
        let ray_origin = ray.position + random_direction() * config.non_focal_offset;
        current_ray.position = ray_origin;
        current_ray.direction = normalize(ray_target - ray_origin);
        color += trace_ray(current_ray);
    }
    color /= f32(config.rays_per_pixel);

    return vec4(color, 1.0);
}
struct DistanceInfo {
    did_hit: bool,
    distance: f32,
}

// rng
var<private> rng_seed: u32 = 0u;
fn initRngSeed(pixel: vec2<f32>) {
    // Compute a seed from pixel coordinates and a frame constant using a simple hash.
    // Given two floats in [0,1), scale them to the full 32-bit range,
    // then mix their bits using a simple hash function.
    // Scale the floats to 32-bit unsigned integer range.
    // (We subtract a tiny epsilon to avoid exactly 1.0.)
    let ai: u32 = u32(clamp(pixel.x + 0.5, 0.0, 0.999999) * 4294967296.0);
    let bi: u32 = u32(clamp(pixel.y + 0.5, 0.0, 0.999999) * 4294967296.0);

    // Combine the two values into a 64-bit value, then mix down to 32 bits.
    // Since WGSL doesn’t have 64-bit integers yet, we can instead combine them via bitwise mixing.
    // One simple approach is to use XOR and a multiplication with a large odd constant.
    var h: u32 = ai ^ (bi * 0x85ebca6bu);
    h = (h ^ (h >> 16u)) * 0x85ebca6bu;
    rng_seed = jenkinsHash(u32(h ^ (h >> 13u)));
}
fn jenkinsHash(input: u32) -> u32 {
  var x = input;
  x += x << 10u;
  x ^= x >> 6u;
  x += x << 3u;
  x ^= x >> 11u;
  x += x << 15u;
  return x;
}
fn random_int() -> u32 {
    let newSeed = rng_seed * 747796405u + 2891336453u;
    rng_seed = newSeed;
    let word = ((newSeed >> ((newSeed >> 28u) + 4u)) ^ newSeed) * 277803737u;
    return (word >> 22u) ^ word;
}
fn random_float() -> f32 {
    return f32(random_int()) / f32(0xffffffffu);
}
fn random_direction() -> vec3<f32> {
    let z: f32 = random_float() * 2.0 - 1.0;
    let theta = random_float() * 2.0 * PI;
    let r = sqrt(1.0 - z * z);
    return vec3<f32> (
        r * cos(theta),
        r * sin(theta),
        z,
    );
}

// actual raytracing beginning
struct Ray {
    position: vec3<f32>,
    direction: vec3<f32>,
    light_color: vec3<f32>,
    actual_color: vec3<f32>,
}
fn trace_ray(ray_: Ray) -> vec3<f32> {
    var ray = ray_;
    for (var i: u32 = 0u; i < config.max_bounces; i++) {
        let hit_info = closest_object(ray);
        if (!hit_info.did_hit) {
            break;
        }
        ray.position += ray.direction * hit_info.distance;
        ray.actual_color += hit_info.object.emission_color * ray.light_color;
        ray.light_color *= max(hit_info.object.base_color, vec3<f32>(0.0, 0.0, 0.0));
        if (all(ray.light_color == vec3<f32>(0.0, 0.0, 0.0))) {
            break;
        }
        let object = hit_info.object;
        let normal = calculate_normal(ray.position, object.object_id, object.object_index);
        ray.direction = random_bounce(ray.direction, normal, object.roughness);
    }
    return ray.actual_color;
}
fn random_bounce(ray_dir: vec3<f32>, surface_normal: vec3<f32>, surface_roughness: f32) -> vec3<f32> {
    let random_dir = random_direction();
    let reflection_dir = ray_dir - surface_normal * 2 * dot(ray_dir, surface_normal);
    let random_to_reflection = reflection_dir - random_dir;
    let reflection_mult = 1.0 - surface_roughness;
    var final_direction = random_dir + random_to_reflection * reflection_mult;

    final_direction = normalize(final_direction);
    if (dot(final_direction, surface_normal) < 0.0) {
        return -final_direction;
    } else {
        return final_direction;
    }
}
struct RayHitInfo {
    did_hit: bool,
    object: Object,
    distance: f32,
}
const NULL_OBJECT: Object = Object(vec3<f32>(0.0, 0.0, 0.0), 0.0, vec3<f32>(0.0, 0.0, 0.0), 0, 0);
fn closest_object(ray: Ray) -> RayHitInfo {
    var res: RayHitInfo = RayHitInfo(false, NULL_OBJECT, -1.0);
    for (var i: u32 = 0u; i < arrayLength(&objects); i++) {
        let current = objects[i];
        let bounding_box = bounding_box(current.object_id, current.object_index);
        if (!bounding_box_intersection(ray, bounding_box)) {
            continue;
        }
        let distance_result = calculate_distance(ray.position, ray.direction, current.object_id, current.object_index);
        if (!distance_result.did_hit) {
            continue;
        }
        let distance = distance_result.distance;
        if (distance <= 0.0) {
            continue;
        }
        if (!res.did_hit || distance < res.distance) {
            res = RayHitInfo(true, current, distance);
        }
    }
    return res;
}
fn bounding_box_intersection(ray: Ray, box: BoundingBox) -> bool {
    if !box.has_box {
        return true;
    }
    let inv_dir = 1.0 / ray.direction;
    // Compute intersections with the slabs for each axis
    let t0s = (box.min - ray.position) * inv_dir;
    let t1s = (box.max - ray.position) * inv_dir;
    // For each axis, determine the min and max intersection distances
    let tmin = max(max(min(t0s.x, t1s.x), min(t0s.y, t1s.y)), min(t0s.z, t1s.z));
    let tmax = min(min(max(t0s.x, t1s.x), max(t0s.y, t1s.y)), max(t0s.z, t1s.z));
    return tmax >= max(tmin, 0.0);
}

// thx ChatGPT
// Function to compute the inverse of a 3x3 matrix.
fn inverse3x3(m: mat3x3<f32>) -> mat3x3<f32> {
    // Since m is column-major, we extract elements as follows:
    let a = m[0][0]; // first column, first row
    let d = m[0][1]; // first column, second row
    let g = m[0][2]; // first column, third row

    let b = m[1][0]; // second column, first row
    let e = m[1][1]; // second column, second row
    let h = m[1][2]; // second column, third row

    let c = m[2][0]; // third column, first row
    let f = m[2][1]; // third column, second row
    let i = m[2][2]; // third column, third row

    // Compute the determinant:
    // det = a*(e*i - f*h) - b*(d*i - f*g) + c*(d*h - e*g)
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);

    // For robustness, you might want to check if 'det' is near zero.
    // Here we assume det is non-zero.

    // Compute the components of the adjugate matrix.
    let A = e * i - f * h;
    let B = f * g - d * i;
    let C = d * h - e * g;

    let D = c * h - b * i;
    let E = a * i - c * g;
    let F = b * g - a * h;

    let G = b * f - c * e;
    let H = c * d - a * f;
    let I = a * e - b * d;

    // Construct the inverse as (1/det) * adjugate.
    return (1/det) * mat3x3<f32>(
        vec3<f32>(A, D, G), // first column of the inverse
        vec3<f32>(B, E, H), // second column
        vec3<f32>(C, F, I)  // third column
    );
}