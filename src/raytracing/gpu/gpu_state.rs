use std::borrow::Cow;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, CommandEncoderDescriptor, Device, FragmentState, PipelineLayout, RenderPassDescriptor, RenderPipeline, ShaderStages, Surface, VertexState, TextureView, Operations, LoadOp, Color, StoreOp, CommandBuffer};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::Camera;
use crate::raytracing::gpu::GpuSerialize;
use crate::raytracing::gpu::object::Object;

const BASE_SHADER: &str = include_str!("base_shader.wgsl");
pub struct GpuState<'a> {
    pub(crate) surface: Surface<'a>,
    pub(crate) device: Device,
    pub(crate) pipeline: RenderPipeline,
    pub(crate) targets: Vec<Option<ColorTargetState>>,
    pub(crate) objects_buffer: Buffer,
    pub(crate) object_buffers: Vec<Buffer>,
    pub(crate) cam_buffer: Buffer,
    pub(crate) aspect_ratio_buffer: Buffer,
}
impl<'a> GpuState<'a> {
    pub fn new(surface: Surface<'a>, device: Device, targets: Vec<Option<ColorTargetState>>, camera: &Camera) -> GpuState<'a> {
        let cam_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Raytracing camera buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: &*camera.serialize(),
        });
        let aspect_ratio_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Raytracing aspect ratio buffer"),
            size: size_of::<f32>() as wgpu::BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let objects_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Object information"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            size: 0,
            mapped_at_creation: false,
        });
        let pipeline = Self::create_pipeline(&device, &*targets, &vec![]);
        println!("gpu state initialized");
        Self {
            surface,
            device,
            targets,
            object_buffers: vec![],
            pipeline,
            cam_buffer,
            aspect_ratio_buffer,
            objects_buffer,
        }
    }
    pub fn destroy(&mut self) {
        for buffer in &mut self.object_buffers {
            buffer.destroy();
        }
        self.object_buffers = vec![];
        self.cam_buffer.destroy();
        self.aspect_ratio_buffer.destroy();
    }
    pub fn get_surface<'b>(&'a self) -> &'b Surface where 'b: 'a {
        &self.surface
    }
    pub fn get_device(&self) -> &Device {
        &self.device
    }
    pub fn render(&self, aspect_ratio: f32, queue: &wgpu::Queue, view: &TextureView) -> CommandBuffer {
        // update aspect ratio buffer
        queue.write_buffer(&self.aspect_ratio_buffer, 0, &aspect_ratio.to_le_bytes());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Raytracing command encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Raytracing Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.create_bind_group_for_builtins(), &[]);
            render_pass.set_bind_group(1, &self.create_bind_group_for_objects(), &[]);
            render_pass.draw(0..3, 0..1);
        }
        encoder.finish()
    }
    pub fn add_object(&mut self, object: Object) {
        self.objects_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Object information"),
            size: self.objects_buffer.size() + size_of::<Object>() as wgpu::BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shape = object.shape.lock().unwrap();
        self.object_buffers.push(
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Raytracing Object"),
                usage: BufferUsages::UNIFORM,
                contents: &*shape.serialize(),
            })
        );
        self.pipeline = Self::create_pipeline(&self.device, &*self.targets, &self.object_buffers);
    }
}
impl<'a> GpuState<'a> {
    fn create_pipeline(device: &Device, targets:  &[Option<ColorTargetState>], object_buffers: &Vec<Buffer>) -> RenderPipeline {
        let pipeline_layout = Self::create_pipeline_layout(device, object_buffers);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raytracing shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(Self::build_shader())),
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raytracing render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        })
    }
    fn create_pipeline_layout(device: &Device, object_buffers: &Vec<Buffer>) -> PipelineLayout {
        let builtins_bind_group_layout = Self::create_bind_group_layout_for_builtins(device);
        let object_bind_group_layout = Self::create_bind_group_layout_for_objects(device, object_buffers);
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("raytracing pipeline layout"),
            bind_group_layouts: &[&builtins_bind_group_layout, &object_bind_group_layout],
            push_constant_ranges: &[],
        })
    }
    fn create_bind_group_layout_for_builtins(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("raytracing builtin bind group layouts"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
            ],
        })
    }
    fn create_bind_group_layout_for_objects(device: &Device, buffers: &Vec<Buffer>) -> BindGroupLayout {
        let entries = buffers.iter()
            .enumerate()
            .map(|(i, buffer)| {
                BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            })
            .collect::<Vec<_>>();
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("raytracing objects bind group layout"),
            entries: &*entries,
        })
    }
    fn build_shader<'b>() -> &'b str {
        BASE_SHADER
    }
    fn create_bind_group_for_builtins(&self) -> BindGroup {
        self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("raytracing builtin bind group"),
            layout: &Self::create_bind_group_layout_for_builtins(&self.device),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.cam_buffer,
                        offset: 0,
                        size: None,
                    })
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.aspect_ratio_buffer,
                        offset: 0,
                        size: None,
                    })
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.objects_buffer,
                        offset: 0,
                        size: None,
                    })
                }
            ],
        })
    }
    fn create_bind_group_for_objects(&self) -> BindGroup {
        let entries = self.object_buffers.iter()
            .enumerate()
            .map(|(i, buffer)| {
                BindGroupEntry {
                    binding: i as u32,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer,
                        offset: 0,
                        size: None,
                    }),
                }
            })
            .collect::<Vec<_>>();
        self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("raytracing objects bind group"),
            layout: &Self::create_bind_group_layout_for_objects(&self.device, &self.object_buffers),
            entries: &*entries,
        })
    }
}