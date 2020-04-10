use crate::states::loading::GameAssets;
use crate::states::States;
use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::primitives::{Quad, Vertex};
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;

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

        let bind_group = render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_ctx.pipelines.textured.1,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
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

        let (mut vertices, indices) = quad.vertices_and_indices([0., 0., 0.], [1., 1., 1., 1.]);

        // TODO: fix scaling
        for vert in &mut vertices {
            for coord in &mut vert.position {
                *coord = *coord * 3.;
            }
        }

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

        rpass.set_pipeline(&render_ctx.pipelines.textured.0);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

impl DebugDrawable for Play {}
