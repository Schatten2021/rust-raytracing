mod buffer;

use crate::raytracing::gpu::gpu_state::buffer::FrequentlyChangedBuffer;
use crate::raytracing::gpu::object::Object;
use crate::raytracing::gpu::GpuSerialize;
use crate::Camera;
use std::borrow::Cow;
use std::fmt::Display;
use wgpu::util::DeviceExt;

const BASE_SHADER: &str = include_str!("base_shader.wgsl");
struct ShapeInfo<'a> {
    buffer: FrequentlyChangedBuffer<'a>,
    distance_function: String,
    normal_function: String,
    struct_fields: Vec<(String, String)>,
    count: usize,
}
impl ShapeInfo<'_> {
    pub fn build_struct(&self, struct_id: usize) -> String{
        let fields = self.struct_fields.iter()
            .map(|f| format!("{}: {},", f.0, f.1))
            .collect::<Vec<_>>()
            .join("\n");
        format!("struct Shape{struct_id} {{
    {fields}
}}")
    }
}

pub(super) struct State<'a> {
    device: wgpu::Device,
    pipeline: wgpu::RenderPipeline,
    targets: Vec<Option<wgpu::ColorTargetState>>,
    object_data: FrequentlyChangedBuffer<'a>,
    objects: Vec<(String, ShapeInfo<'a>)>, // not a hashmap to preserve order
    cam_buffer: FrequentlyChangedBuffer<'a>,
    aspect_ratio_buffer: FrequentlyChangedBuffer<'a>,
}
impl<'a> State<'a> {
    pub fn new(device: &wgpu::Device, targets: Vec<Option<wgpu::ColorTargetState>>, camera: &Camera) -> Self {
        let cam_buffer = FrequentlyChangedBuffer::new_init(device, Some("raytracing camera buffer"), camera.serialize());
        let aspect_ratio_buffer = FrequentlyChangedBuffer::new_init(device, Some("raytracing aspect ratio buffer"), 0f32.to_le_bytes().to_vec());
        let pipeline = Self::create_pipeline(device, &targets, &Vec::new());
        let object_data = FrequentlyChangedBuffer::new(device, Some("raytracing object data"));
        let device = device.clone();
        Self {
            device,
            targets,
            pipeline,
            cam_buffer,
            aspect_ratio_buffer,
            object_data,
            objects: Vec::new(),
        }
    }
    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn add_object(&mut self, object: Object) {
        let shape = object.shape.lock().unwrap();
        let r#type = shape.object_type();
        let info = match self.objects.iter_mut().find(|(id, _)| id == &r#type) {
            Some((_, info)) => info,
            None => {
                self.objects.push((r#type.clone(), ShapeInfo {
                    buffer: FrequentlyChangedBuffer::new(&self.device, Some("raytracing object")),
                    distance_function: shape.distance_code(),
                    normal_function: shape.normal_calculation_code(),
                    struct_fields: shape.struct_fields(),
                    count: 0,
                }));
                &mut self.objects.iter_mut().find(|(id, _)| id == &r#type).unwrap().1
            }
        };
        let object_index = info.count;
        info.buffer.append(shape.serialize());
        info.count += 1;
        let type_id = self.objects.iter().enumerate().find(|(i, (id, _))| id == &r#type).unwrap().0;
        self.object_data.append(object.gpu_serialize(type_id as u32, object_index as u32));
        self.pipeline = Self::create_pipeline(&self.device, &self.targets, &self.objects);
    }
    pub fn render(&mut self, aspect_ratio: f32, queue: &wgpu::Queue, view: &wgpu::TextureView) {
        queue.write_buffer(&self.aspect_ratio_buffer.get_updated_buffer(queue), 0, &aspect_ratio.to_le_bytes());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("raytracing render pass encoder"),
        });
        let bind_group0 = self.create_bind_group_for_builtins(queue);
        let bind_group1 = self.create_bind_group_for_objects(queue);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("raytracing render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group0, &[]);
            render_pass.set_bind_group(1, &bind_group1, &[]);
            render_pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }
}
impl State<'_> {
    fn create_pipeline(device: &wgpu::Device, targets: &Vec<Option<wgpu::ColorTargetState>>, objects: &Vec<(String, ShapeInfo)>) -> wgpu::RenderPipeline {
        let layout = Self::create_pipeline_layout(device, objects.len());
        let shader: String = Self::create_shader(objects);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raytracing shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&*shader)),
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raytracing render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &**targets,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        })
    }
    fn create_pipeline_layout(device: &wgpu::Device, object_buffer_count: usize) -> wgpu::PipelineLayout {
        let builtins_bind_group_layout = Self::creat_builtin_bind_group_layout(device);
        let object_bind_group_layout = Self::create_bind_group_layout_for_objects(device, object_buffer_count);
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("raytracing pipeline layout"),
            bind_group_layouts: &[&builtins_bind_group_layout, &object_bind_group_layout],
            push_constant_ranges: &[],
        })
    }
    fn creat_builtin_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raytracing builtin bind group layouts"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
            ],
        })
    }
    fn create_bind_group_layout_for_objects(device: &wgpu::Device, buf_count: usize) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raytracing objects bind group layout"),
            entries: &(0..buf_count)
                .map(|i| {
                    wgpu::BindGroupLayoutEntry {
                        binding: i as u32,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                }).collect::<Vec<_>>(),
        })
    }
    fn create_shader(objects: &Vec<(String, ShapeInfo)>) -> String {
        let (structs, uniforms, distance_functions, normal_functions) = objects.iter()
            .map(|v| &v.1)
            .enumerate()
            .map(|(i, info)| {
                (
                    format!("struct Shape{i} {{\
    {}
}}", info.struct_fields.iter()
                        .map(|(name, r#type)| format!("{name}: {type},"))
                        .collect::<Vec<_>>()
                        .join("\n")
                    ),
                    format!("@group(1)
@binding({i})
var<storage, read> shape_{i}: array<Shape{i}>;"),
                    format!("fn distance_shape_{i}(ray_position: vec3<f32>, ray_direction: vec3<f32>, index: u32) -> DistanceInfo {{
    let current = shape_{i}[index];
    {}
}}", info.distance_function),
                    format!("fn normal_shape_{i}(world_position: vec3<f32>, index: u32) -> vec3<f32> {{
    let current = shape_{i}[index];
    {}
}}", info.normal_function),
                )
            })
            .fold((String::new(), String::new(), String::new(), String::new()), |a, b| {
                (a.0 + &b.0,
                 a.1 + &b.1,
                 a.2 + &b.2,
                a.3 + &b.3)
            });
        let dst_func = format!(
            "fn calculate_distance(ray_pos: vec3<f32>, ray_dir: vec3<f32>, object_id: u32, object_index: u32) -> DistanceInfo {{\
                switch (object_id) {{
                    {cases}
                    default: {{return DistanceInfo(false, 0.0);}}
                }}
            }}",
            cases=(0..objects.len())
                .map(|i| format!("case {i}u: {{return distance_shape_{i}(ray_pos, ray_dir, object_index);}}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let normal_func = format!(
            "fn calculate_normal(ray_pos: vec3<f32>, object_id: u32, object_index: u32) -> vec3<f32> {{\
                switch (object_id) {{
                    {}
                    default: {{return vec3<f32>(1.0, 0.0, 0.0);}}
                }}
            }}",
            (0..objects.len())
                .map(|i| format!("case {i}u: {{return normal_shape_{i}(ray_pos, object_index);}}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let shader = format!("{BASE_SHADER}
//
// GENERATED CODE
//
{dst_func}
{normal_func}

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
{structs}
//
// UNIFORMS
//
{uniforms}");
        // println!("{}", shader);
        shader
    }
}
impl State<'_> {
    fn create_bind_group_for_builtins(&mut self, queue: &wgpu::Queue) -> wgpu::BindGroup {
        macro_rules! entry {
            ($idx:expr, $buf:ident) => {
                wgpu::BindGroupEntry {
                    binding: $idx,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.$buf.get_updated_buffer(queue),
                        offset: 0,
                        size: None,
                    })
                }
            };
        }
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("raytracing builtin bind group"),
            layout: &Self::creat_builtin_bind_group_layout(&self.device),
            entries: &[
                entry!(0, cam_buffer),
                entry!(1, aspect_ratio_buffer),
                entry!(2, object_data),
            ]
        })
    }
    fn create_bind_group_for_objects(&mut self, queue: &wgpu::Queue) -> wgpu::BindGroup {
        macro_rules! entry {
            ($idx:expr, $buf:expr) => {
                wgpu::BindGroupEntry {
                    binding: $idx,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: $buf.get_updated_buffer(queue),
                        offset: 0,
                        size: None,
                    })
                }
            };
        }
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("raytracing objects bind group"),
            layout: &Self::create_bind_group_layout_for_objects(&self.device, self.objects.len()),
            entries: &*self.objects.iter_mut()
                .map(|i| &mut i.1)
                .enumerate()
                .map(|(i, obj)| entry!(i as u32, &mut obj.buffer))
                .collect::<Vec<_>>()
        })
    }
}