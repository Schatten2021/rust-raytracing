use crate::raytracing::gpu::gpu_state::State;
use crate::raytracing::gpu::object::Object;
use crate::Camera;
use wgpu::{ColorTargetState, Device, Queue, TextureView};

pub struct Scene<'a> {
    camera: Camera,
    objects: Vec<Object>,
    state: State<'a>
}
impl<'a> Scene<'a> {
    pub fn new(camera: Camera, device: &Device, targets: Vec<Option<ColorTargetState>>) -> Self {
        let state = State::new(&device, targets, &camera);
        Self {
            camera,
            objects: Vec::new(),
            state,
        }
    }
    pub fn get_device(&self) -> &Device {
        self.state.get_device()
    }
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object.clone());
        self.state.add_object(object);
    }
    pub fn render(&mut self, view: &TextureView, aspect_ratio: f32, queue: &Queue) {
        self.state.render(aspect_ratio, queue, view)//, &self.objects)
    }
}