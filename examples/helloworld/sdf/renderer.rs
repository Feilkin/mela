//! :)

use crate::sdf::{SdfObject, SdfShape};
use bytemuck::{Pod, Zeroable};
use mela::components::Transform;
use mela::ecs::{maybe_changed, IntoQuery, World};
use mela::gfx::MiddlewareRenderer;
use mela::na as nalgebra;
use mela::na::vector;
use mela::wgpu::util::BufferInitDescriptor;
use mela::wgpu::{
    util::StagingBelt, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsage, ColorTargetState, CommandEncoder,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode, FragmentState,
    ImageCopyTexture, RenderPass, RenderPipeline, RenderPipelineDescriptor, Sampler, ShaderStage,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    VertexAttribute, VertexBufferLayout,
};
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ComputePassDescriptor,
    PipelineLayoutDescriptor, SamplerDescriptor, TextureUsage, TextureViewDescriptor,
    TextureViewDimension, VertexFormat, VertexState,
};

const WORLD_SDF_SIZE: [u32; 3] = [256, 256, 128];
const MAX_BRUSH_INSTANCES: usize = 500;

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct SdfData {
    translation: [f32; 3],
    shape: u32,
    shape_data: [f32; 4],
}

impl From<(&Transform, &SdfObject)> for SdfData {
    fn from((transform, obj): (&Transform, &SdfObject)) -> Self {
        let (shape, shape_data) = match &obj.shape {
            &SdfShape::Ball(radius) => (1, [radius, 0., 0., obj.smoothing]),
            &SdfShape::Cuboid(hx, hy, hz) => (2, [hx, hy, hz, obj.smoothing]),
        };

        SdfData {
            translation: transform.0.translation.into(),
            shape,
            shape_data,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct BrushInstances {
    count: u32,
    __padding: [u32; 3],
    instances: [SdfData; MAX_BRUSH_INSTANCES],
}

impl From<&[SdfData]> for BrushInstances {
    fn from(data: &[SdfData]) -> Self {
        let count = data.len() as u32;
        assert!(count <= MAX_BRUSH_INSTANCES as u32);
        let mut instances = [SdfData::default(); MAX_BRUSH_INSTANCES];

        for (i, instance) in data.iter().enumerate() {
            instances[i] = instance.clone();
        }

        BrushInstances {
            count,
            __padding: [0, 0, 0],
            instances,
        }
    }
}

pub struct SdfRenderer {
    bake_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    brush_instance_buffer: Buffer,
    world_sdf_texture: Texture,
    world_data_sampler: Sampler,
    vertex_buffer: Buffer,
    render_bindings: BindGroup,
    camera_buffer: Buffer,
    sdf_gpu_objects: Vec<SdfData>, // this is reused for performance
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 4],
}

fn vertices() -> Vec<u8> {
    fn vertex(x: f32, y: f32) -> Vertex {
        Vertex {
            pos: [x, y, 0., 1.],
        }
    }

    bytemuck::bytes_of(&[
        vertex(-1., -1.),
        vertex(1., -1.),
        vertex(-1., 1.),
        vertex(1., -1.),
        vertex(1., 1.),
        vertex(-1., 1.),
    ])
    .to_vec()
}

impl MiddlewareRenderer for SdfRenderer {
    fn new(device: &Device, texture_format: &TextureFormat, _screen_size: [f32; 2]) -> Self {
        let module = device.create_shader_module(&wgpu::include_wgsl!("./bake.wgsl"));
        let render_shaders = device.create_shader_module(&wgpu::include_wgsl!("./render.wgsl"));

        let camera = mela::na::Matrix4::look_at_lh(
            &vector![0_f32, -256., 128.].into(),
            &vector![0., 0., 32.].into(),
            &vector![0., 0., 1.].into(),
        )
        .try_inverse()
        .unwrap();

        let camera_array: [[f32; 4]; 4] = camera.into();

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("SdfRenderer::camera_buffer"),
            contents: bytemuck::bytes_of(&camera_array),
            usage: BufferUsage::UNIFORM,
        });

        let world_sdf_texture = device.create_texture(&TextureDescriptor {
            label: Some("SdfRenderer::world_sdf_texture"),
            size: Extent3d {
                width: WORLD_SDF_SIZE[0],
                height: WORLD_SDF_SIZE[1],
                depth_or_array_layers: WORLD_SDF_SIZE[2],
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::R32Float,
            usage: TextureUsage::STORAGE | TextureUsage::SAMPLED,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("SdfRenderer::render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("SdfRenderer::render_pipeline_layout"),
                bind_group_layouts: &[&device.create_bind_group_layout(
                    &BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Texture {
                                    sample_type: TextureSampleType::Float { filterable: true },
                                    view_dimension: TextureViewDimension::D3,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Sampler {
                                    filtering: true,
                                    comparison: false,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 2,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    },
                )],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: &render_shaders,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: Default::default(),
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &render_shaders,
                entry_point: "fs_main",
                targets: &[ColorTargetState::from(texture_format.clone())],
            }),
        });

        let world_data_sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        SdfRenderer {
            bake_pipeline: device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("SdfRenderer::bake_pipeline"),
                layout: None,
                module: &module,
                entry_point: "bake",
            }),
            brush_instance_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("SdfRenderer::brush_instance_buffer"),
                size: std::mem::size_of::<BrushInstances>() as u64,
                usage: BufferUsage::COPY_DST | BufferUsage::UNIFORM,
                mapped_at_creation: false,
            }),
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("SdfRenderer::vertex_buffer"),
                contents: vertices().as_slice(),
                usage: BufferUsage::VERTEX,
            }),
            render_bindings: device.create_bind_group(&BindGroupDescriptor {
                label: Some("SdfRenderer::render_bindings"),
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&world_sdf_texture.create_view(
                            &TextureViewDescriptor {
                                label: None,
                                format: Some(TextureFormat::R32Float),
                                dimension: None,
                                aspect: Default::default(),
                                base_mip_level: 0,
                                mip_level_count: None,
                                base_array_layer: 0,
                                array_layer_count: None,
                            },
                        )),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&world_data_sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(camera_buffer.as_entire_buffer_binding()),
                    },
                ],
            }),
            render_pipeline,
            world_sdf_texture,
            sdf_gpu_objects: Vec::new(),
            world_data_sampler,
            camera_buffer,
        }
    }

    #[profiling::function]
    fn prepare(
        &mut self,
        world: &mut World,
        device: &Device,
        staging_belt: &mut StagingBelt,
        command_encoder: &mut CommandEncoder,
    ) {
        // collect SDF objects and their transforms so we can bake them
        let mut query = <(&Transform, &SdfObject)>::query();

        self.sdf_gpu_objects.clear();

        for (transform, sdf_object) in query.iter_mut(world) {
            self.sdf_gpu_objects.push((transform, sdf_object).into());
        }

        let brush_instances: BrushInstances = (self.sdf_gpu_objects.as_slice()).into();

        staging_belt
            .write_buffer(
                command_encoder,
                &self.brush_instance_buffer,
                0,
                unsafe { NonZeroU64::new_unchecked(std::mem::size_of::<BrushInstances>() as u64) },
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&brush_instances));

        let bake_binds = device.create_bind_group(&BindGroupDescriptor {
            label: Some("SdfRenderer::bake_binds"),
            layout: &self.bake_pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.brush_instance_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &self
                            .world_sdf_texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        {
            // bake world SDF data into texture with compute shader
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("SdfRenderer bake pass"),
            });

            pass.set_pipeline(&self.bake_pipeline);
            pass.set_bind_group(0, &bake_binds, &[]);
            pass.insert_debug_marker("bake SDF data");
            pass.dispatch(
                WORLD_SDF_SIZE[0] / 8,
                WORLD_SDF_SIZE[1] / 8,
                WORLD_SDF_SIZE[2] / 16,
            );
        }
    }

    #[profiling::function]
    fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.render_bindings, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
