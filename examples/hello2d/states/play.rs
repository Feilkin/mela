use crate::states::loading::GameAssets;
use crate::states::States;
use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;
use nalgebra::{Vector2, Vector3};

pub struct Play {
    assets: GameAssets,
    bind_group: wgpu::BindGroup,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let texture_view = assets
            .textures
            .get("spritesheet")
            .unwrap()
            .create_default_view();

        let sampler = render_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: Default::default(),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Never,
        });

        let projection = nalgebra::Matrix4::new_nonuniform_scaling(&Vector3::new(1., 16./9., 1.))
            .append_translation(&Vector3::new(-1., -1., 0.));

        let view = nalgebra::Matrix4::new_translation(&Vector3::new(0., 0., 0.));
        let model = nalgebra::Matrix4::new_scaling(1.);

        let transformations = MVP {
            model: model.into(),
            view: view.into(),
            proj: projection.into(),
        };

        let transforms_buffer = render_ctx
            .device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[transformations]);

        let bind_group = render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_ctx.pipelines.pixel.1,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &transforms_buffer,
                            range: 0..std::mem::size_of::<MVP>() as u64
                        }
                    }
                ],
            });

        Play { assets, bind_group }
    }
}

impl State for Play {
    type Wrapper = States;

    fn name(&self) -> &str {
        "Play"
    }

    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        States::Play(self)
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        let quad = Quad::new(17., 17., 16., 16., 543., 543.);

        let (mut vertices, indices) = quad.vertices_and_indices2d([0., 0.], [1., 1., 1., 1.]);


        let vertex_buf = render_ctx
            .device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        let index_buf = render_ctx
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);

        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_ctx.frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

impl DebugDrawable for Play {}
