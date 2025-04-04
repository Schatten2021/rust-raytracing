use crate::raytracing::gpu::gpu_state::GpuState;
use crate::raytracing::gpu::object::Object;
use crate::Camera;
use wgpu::{ColorTargetState, Device, Queue, Surface, TextureView};

pub struct Scene<'a> {
    camera: Camera,
    objects: Vec<Object>,
    state: GpuState<'a>
}
impl<'a> Scene<'a> {
    pub fn new(camera: Camera, surface: Surface<'a>, device: Device, targets: Vec<Option<ColorTargetState>>) -> Self {
        let state = GpuState::new(surface, device, targets, &camera);
        Self {
            camera,
            objects: Vec::new(),
            state,
        }
    }
    pub fn get_surface<'b>(&'a self) -> &'b Surface where 'b: 'a {
        self.state.get_surface()
    }
    pub fn get_device(&self) -> &Device {
        self.state.get_device()
    }
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object.clone());
        self.state.add_object(object);
    }
    pub fn render(&self, view: &TextureView, aspect_ratio: f32, queue: &Queue) -> wgpu::CommandBuffer {
        self.state.render(aspect_ratio, queue, view)//, &self.objects)
    }
}