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
    //vec3<f32> requires a 16 bit alignement, that's why those above are where they are.
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> aspect_ratio: f32;

@group(0)
@binding(2)
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
    output.uv = position * 0.5; // + vec2<f32>(0.5, 0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let angle = input.uv * vec2<f32>(camera.fov, camera.fov / aspect_ratio);
    let cam_space_dir: vec3<f32> = vec3<f32>(sin(angle), cos(angle.x) * cos(angle.y));

    let cam_space_forward: vec3<f32> = normalize(camera.dir);
    let cam_space_right: vec3<f32> = cross(cam_space_forward, vec3<f32>(0.0, 0.0, 1.0));
    let cam_space_up: vec3<f32> = cross(cam_space_forward, cam_space_right);
    let to_cam_space_mat: mat3x3<f32> = mat3x3<f32>(cam_space_right, cam_space_up, cam_space_forward);

    let ray_dir = inverse3x3(to_cam_space_mat) * cam_space_dir;

    let ray = Ray(camera.pos, ray_dir, vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.0, 0.0, 0.0));

    var color = vec4(trace_ray(ray), 1.0);
//    let color = vec4(camera.fov, aspect_ratio / 2, 0.0, 1.0);
//    let object_length: u32 = arrayLength(&objects);
//    let object_length_float: f32 = f32(object_length);
//    color = vec4(objects[0].base_color, 0.0);
//    color = vec4(camera.dir, 0.0);
    return color;
}
struct DistanceInfo {
    did_hit: bool,
    distance: f32,
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
    for (var i: u32 = 0u; i < 10u; i++) {
        let hit_info = closest_object(ray);
        if (!hit_info.did_hit) {
            break;
        }
        ray.position += ray.direction * hit_info.distance;
        ray.actual_color += hit_info.object.emission_color * ray.light_color;
        ray.light_color *= hit_info.object.base_color;
        if (all(ray.light_color == vec3<f32>(0.0, 0.0, 0.0))) {
            break;
        }
    }
    return ray.actual_color;
}
struct RayHitInfo {
    did_hit: bool,
    object: Object,
    distance: f32,
}
const NULL_OBJECT: Object = Object(vec3<f32>(0.0, 0.0, 0.0), 0.0, vec3<f32>(0.0, 0.0, 0.0), 0);
fn closest_object(ray: Ray) -> RayHitInfo {
    var res: RayHitInfo = RayHitInfo(false, NULL_OBJECT, -1.0);
    for (var i: u32 = 0u; i < arrayLength(&objects); i++) {
        let current = objects[i];
        let distance_result = calculate_distance(ray.position, ray.direction, current.object_id);
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