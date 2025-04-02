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
    var color = vec4(cam_space_dir, 1.0);
//    let color = vec4(camera.fov, aspect_ratio / 2, 0.0, 1.0);
    let object_length: u32 = arrayLength(&objects);
    let object_length_float: f32 = f32(object_length);
//    color = vec4(objects[0].base_color, 0.0);
    return color;
}
struct DistanceInfo {
    did_hit: bool,
    distance: vec3<f32>,
}