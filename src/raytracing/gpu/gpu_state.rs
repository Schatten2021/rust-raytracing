use std::borrow::Cow;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, CommandEncoderDescriptor, Device, FragmentState, PipelineLayout, RenderPassDescriptor, RenderPipeline, ShaderStages, Surface, VertexState, TextureView, Operations, LoadOp, Color, StoreOp, CommandBuffer, Label};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::Camera;
use crate::raytracing::gpu::GpuSerialize;
use crate::raytracing::gpu::object::Object;

const BASE_SHADER: &str = include_str!("base_shader.wgsl");
struct ChangeableBuffer<'a> {
    device: Device,
    label: Label<'a>,
    buffer: Buffer,
    data: Vec<u8>,
    requires_update: bool,
}
impl<'a> ChangeableBuffer<'a> {
    pub(crate) fn new(device: &Device, label: Label<'a>) -> Self {
        Self {
            device: device.clone(),
            label: label.clone(),
            buffer: device.create_buffer(&BufferDescriptor {
                label,
                size: 0,
                usage: BufferUsages::UNIFORM | BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            data: vec![],
            requires_update: false,
        }
    }
    pub(crate) fn add_data(&mut self, data: impl Iterator<Item = u8>) {
        self.data.extend(data);
        self.requires_update = true;
    }
    pub(crate) fn get_updated_buffer(&mut self) -> &Buffer {
        if self.requires_update {
            self.buffer.destroy();
            self.buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: self.label,
                contents: &*self.data,
                usage: BufferUsages::UNIFORM | BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            self.requires_update = false;
        }
        &self.buffer
    }
    pub(crate) fn destroy(&self) -> () {
        self.buffer.destroy();
    }
    pub(crate) fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.requires_update = true;
    }
    pub(crate) fn insert_data(&mut self, data: Vec<u8>, start_index: usize) -> () {
        if start_index + data.len() > self.data.len() {
            self.data.resize(start_index + data.len(), 0);
        }
        self.data[start_index..start_index + data.len()] = *data;
    }
}

pub struct GpuState<'a> {
    device: Device,
    pipeline: RenderPipeline,
    targets: Vec<Option<ColorTargetState>>,
    default_object_data: ChangeableBuffer<'a>,
    object_buffers: Vec<Buffer>,
    cam_buffer: Buffer,
    aspect_ratio_buffer: Buffer,
    distance_functions: Vec<String>,
    normal_functions: Vec<String>,
    struct_fields: Vec<Vec<(String, String)>>,
}
impl<'a> GpuState<'a> {
    pub fn new(device: &Device, targets: Vec<Option<ColorTargetState>>, camera: &Camera) -> GpuState<'a> {
        let device = Device::clone(device);
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
        let pipeline = Self::create_pipeline(&device, &*targets, &vec![], &vec![], &vec![], &vec![]);
        let default_object_data = ChangeableBuffer::new(&device, Some("raytracing object data"));
        // println!("gpu state initialized");
        Self {
            device,
            targets,
            pipeline,
            cam_buffer,
            aspect_ratio_buffer,
            default_object_data,
            object_buffers: vec![],
            distance_functions: vec![],
            normal_functions: vec![],
            struct_fields: vec![],
        }
    }
    pub fn destroy(&mut self) {
        for buffer in &mut self.object_buffers {
            buffer.destroy();
        }
        self.object_buffers = vec![];
        self.cam_buffer.destroy();
        self.aspect_ratio_buffer.destroy();
        self.default_object_data.destroy();
    }
    pub fn get_device(&self) -> &Device {
        &self.device
    }
    pub fn render(&mut self, aspect_ratio: f32, queue: &wgpu::Queue, view: &TextureView) -> CommandBuffer {
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
        self.default_object_data.add_data(object.gpu_serialize(self.object_buffers.len() as u32).into_iter());
        let shape = object.shape.lock().unwrap();
        self.object_buffers.push(
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Raytracing Object"),
                usage: BufferUsages::UNIFORM,
                contents: &*shape.serialize(),
            })
        );
        self.distance_functions.push(shape.distance_code());
        self.normal_functions.push(shape.normal_calculation_code());
        self.struct_fields.push(shape.struct_fields());
        self.pipeline = Self::create_pipeline(&self.device,
                                              &*self.targets,
                                              &self.object_buffers,
                                              &self.distance_functions,
                                              &self.normal_functions,
                                              &self.struct_fields);
    }
}
impl<'a> GpuState<'a> {
    fn create_pipeline(device: &Device,
                       targets:  &[Option<ColorTargetState>],
                       object_buffers: &Vec<Buffer>,
                       distance_functions: &Vec<String>,
                       normal_functions: &Vec<String>,
                       structs_fields: &Vec<Vec<(String, String)>>,) -> RenderPipeline {
        let pipeline_layout = Self::create_pipeline_layout(device, object_buffers);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raytracing shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&*Self::build_shader(distance_functions, normal_functions, structs_fields))),
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
    fn build_shader(distance_functions: &Vec<String>,
                        normal_functions: &Vec<String>,
                        structs_fields: &Vec<Vec<(String, String)>>,) -> String {
        let distance_functions = distance_functions.iter()
            .enumerate()
            .map(|(i, distance_function)| {
                format!("fn distance_object_{i}(ray_position: vec3<f32>, ray_direction: vec3<f32>) -> DistanceInfo {{\
                let current = object_{i};\
                {distance_function}\
                }}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let normal_functions = normal_functions.iter()
            .enumerate()
            .map(|(i, normal_function)| {
                format!("fn normal_object_{i}(world_position: vec3<f32>) -> vec3<f32> {{\
                let current = object_{i};\
                {normal_function}\
                }}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let struct_definitions = structs_fields.iter()
            .enumerate()
            .map(|(i, struct_fields)| {
                format!(
                    "struct ObjectStruct{i} {{{}}}",
                    struct_fields.iter()
                        .map(|(name, r#type)| {
                            format!("\t{name}: {type},")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let uniform_definitions = (0..structs_fields.len())
            .map(|i| {
                format!("@group(1)
@binding({i})
var<uniform> object_{i}: ObjectStruct{i};")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let distance_function = format!(
            "fn calculate_distance(ray_pos: vec3<f32>, ray_direction: vec3<f32>, object_id: u32) -> DistanceInfo {{\
                switch (object_id) {{\
                    {cases}
                    default: {{return DistanceInfo(false, 0.0);}}
                }}
            }}",
            cases = (0..structs_fields.len())
                .map(|i| format!("case {i}u: {{return distance_object_{i}(ray_pos, ray_direction);}}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let normal_function = format!(
            "fn calculate_normal(world_pos: vec3<f32>, object_id: u32) -> vec3<f32> {{\
                switch (object_id) {{\
                    {cases}
                    default: {{return vec3<f32>(1.0,0.0,0.0);}}
                }}
            }}",
            cases = (0..structs_fields.len())
                .map(|i| format!("case {i}u: {{return normal_object_{i}(world_pos);}}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let shader = format!("{BASE_SHADER}

//
// DISTANCE FUNCTIONS
//
{distance_functions}
//
// NORMAL FUNCTIONS
//
{normal_functions}
//
// STRUCTS
//
{struct_definitions}
//
// UNIFORMS
//
{uniform_definitions}
//
// DISTANCE FUNCTIONS
//
{distance_function}
//
// NORMAL FUNCTIONS
//
{normal_function}");
        // println!("{}", shader);
        shader
    }
    fn create_bind_group_for_builtins(&mut self) -> BindGroup {
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
                        buffer: &self.default_object_data.get_updated_buffer(),
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