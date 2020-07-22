//! Shadow mapping pass

use crate::gfx::light::LightData;
use crate::gfx::pass::Pass;
use crate::gfx::{Mesh, RenderContext, Scene};
use std::sync::Arc;

pub struct ShadowPass {
    light_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    shadow_sampler: wgpu::Sampler,
    shadow_view: wgpu::TextureView,
    light_views: Vec<wgpu::TextureView>,
}

impl ShadowPass {
    pub const MAX_LIGHTS: usize = 256;
    pub const SHADOW_RES: u32 = 1024;

    pub fn new(render_ctx: &mut RenderContext) -> ShadowPass {
        let (pipeline, light_bind_group_layout, model_bind_group_layout) =
            ShadowPass::pipeline(render_ctx);
        let (shadow_sampler, shadow_view, light_views) =
            ShadowPass::prepare_shadow_views(render_ctx);

        ShadowPass {
            light_bind_group_layout,
            model_bind_group_layout,
            pipeline,
            shadow_sampler,
            shadow_view,
            light_views,
        }
    }

    pub fn shadow_view(&self) -> (&wgpu::TextureView, &wgpu::Sampler) {
        (&self.shadow_view, &self.shadow_sampler)
    }

    fn pipeline(
        render_ctx: &mut RenderContext,
    ) -> (
        wgpu::RenderPipeline,
        wgpu::BindGroupLayout,
        wgpu::BindGroupLayout,
    ) {
        let vs_source = include_bytes!("../../../assets/shader/shadow.vert.spv");
        let fs_source = include_bytes!("../../../assets/shader/shadow.frag.spv");

        let vs_module = render_ctx
            .device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs_source[..])).unwrap());
        let fs_module = render_ctx
            .device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs_source[..])).unwrap());

        let light_bind_group_layout =
            render_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }],
                    label: None,
                });

        let model_bind_group_layout =
            render_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }],
                    label: None,
                });

        let pipeline_layout =
            render_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&light_bind_group_layout, &model_bind_group_layout],
                });

        (
            render_ctx
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    layout: &pipeline_layout,
                    vertex_stage: wgpu::ProgrammableStageDescriptor {
                        module: &vs_module,
                        entry_point: "main",
                    },
                    fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                        module: &fs_module,
                        entry_point: "main",
                    }),
                    rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::Back,
                        depth_bias: 2,
                        depth_bias_slope_scale: 2.0,
                        depth_bias_clamp: 0.0,
                    }),
                    primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                    color_states: &[],
                    depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                        stencil_read_mask: 0,
                        stencil_write_mask: 0,
                    }),
                    sample_count: 1,
                    sample_mask: 0,
                    alpha_to_coverage_enabled: false,
                    vertex_state: wgpu::VertexStateDescriptor {
                        index_format: wgpu::IndexFormat::Uint16,
                        vertex_buffers: &[wgpu::VertexBufferDescriptor {
                            stride: 3 * 4,
                            step_mode: wgpu::InputStepMode::Vertex,
                            attributes: &[wgpu::VertexAttributeDescriptor {
                                offset: 0,
                                format: wgpu::VertexFormat::Float3,
                                shader_location: 0,
                            }],
                        }],
                    },
                }),
            light_bind_group_layout,
            model_bind_group_layout,
        )
    }

    fn light_bind_group(
        &self,
        light: &LightData,
        render_ctx: &mut RenderContext,
    ) -> wgpu::BindGroup {
        use zerocopy::AsBytes;

        let transforms_buffer = render_ctx
            .device
            .create_buffer_with_data(&light.view_matrix.as_bytes(), wgpu::BufferUsage::UNIFORM);

        render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.light_bind_group_layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_buffer,
                        range: 0..std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                    },
                }],
                label: None,
            })
    }

    fn model_bind_group(&self, mesh: &dyn Mesh, render_ctx: &mut RenderContext) -> wgpu::BindGroup {
        // TODO: get rid of zerobytes
        use zerocopy::AsBytes;

        #[derive(AsBytes)]
        #[repr(C)]
        struct ModelData {
            transform: [[f32; 4]; 4],
            material: u32,
        }

        let model_data = ModelData {
            transform: mesh.transformation().into(),
            material: mesh.material() as u32,
        };

        let model_buffer = render_ctx
            .device
            .create_buffer_with_data(&model_data.as_bytes(), wgpu::BufferUsage::UNIFORM);

        render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.model_bind_group_layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &model_buffer,
                        range: 0..std::mem::size_of::<ModelData>() as u64,
                    },
                }],
                label: None,
            })
    }

    fn prepare_shadow_views(
        render_ctx: &mut RenderContext,
    ) -> (wgpu::Sampler, wgpu::TextureView, Vec<wgpu::TextureView>) {
        let shadow_sampler = render_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        let shadow_texture = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: ShadowPass::SHADOW_RES,
                height: ShadowPass::SHADOW_RES,
                depth: 1,
            },
            array_layer_count: ShadowPass::MAX_LIGHTS as u32,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let shadow_view = shadow_texture.create_default_view();
        let light_views = (0..ShadowPass::MAX_LIGHTS)
            .map(|i| {
                shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    format: wgpu::TextureFormat::Depth32Float,
                    dimension: wgpu::TextureViewDimension::D2,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: i as u32,
                    array_layer_count: 1,
                })
            })
            .collect();

        (shadow_sampler, shadow_view, light_views)
    }
}

impl<S> Pass<S> for ShadowPass
where
    S: Scene,
{
    fn render(&self, scene: &S, render_ctx: &mut RenderContext) -> () {
        // collect buffers and make bind groups
        let meshes = scene.meshes();
        let (lower_bound, upper_bound) = meshes.size_hint();
        let mut mesh_render_data = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

        struct MeshData {
            index_buffer: Arc<wgpu::Buffer>,
            index_offset: u64,
            index_size: u64,
            index_count: u32,
            vertex_buffer: (Arc<wgpu::Buffer>, u64, u64),
            bind_group: wgpu::BindGroup,
        }

        for mesh in meshes {
            let (index_buffer, index_offset, index_size) = mesh.index_buffer();
            let index_count = (index_size / 2) as u32; // TODO: implement properly
            let vertex_buffer = mesh.positions_buffer();

            let bind_group = self.model_bind_group(mesh, render_ctx);

            mesh_render_data.push(MeshData {
                index_buffer,
                index_offset,
                index_size,
                index_count,
                vertex_buffer,
                bind_group,
            });
        }

        for (i, light) in scene.lights().iter().enumerate() {
            let light_bind_group = self.light_bind_group(light, render_ctx);

            let mut rpass = render_ctx
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.light_views[i],
                            depth_load_op: wgpu::LoadOp::Clear,
                            depth_store_op: wgpu::StoreOp::Store,
                            clear_depth: 1.0,
                            stencil_load_op: wgpu::LoadOp::Clear,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_stencil: 0,
                        },
                    ),
                });

            rpass.set_pipeline(&self.pipeline);

            rpass.set_bind_group(0, &light_bind_group, &[]);

            for mesh in &mesh_render_data {
                rpass.set_bind_group(1, &mesh.bind_group, &[]);

                rpass.set_index_buffer(
                    mesh.index_buffer.as_ref(),
                    mesh.index_offset,
                    mesh.index_size,
                );

                rpass.set_vertex_buffer(
                    i as u32,
                    &mesh.vertex_buffer.0,
                    mesh.vertex_buffer.1,
                    mesh.vertex_buffer.2,
                );

                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
    }
}
