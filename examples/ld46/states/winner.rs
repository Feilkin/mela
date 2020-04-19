//! game over states

use crate::states::loading::GameAssets;
use crate::states::{Loading, Play, States};
use mela::asset::{Asset, AssetState, Bytes};
use mela::debug::{DebugContext, DebugDrawable};
use mela::game::IoState;
use mela::gfx::primitives::Quad;
use mela::gfx::{RenderContext, Texture};
use mela::state::State;
use std::time::Duration;

pub struct Winner {
    background: Texture,
    background_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    buffers: (wgpu::Buffer, wgpu::Buffer),
}

impl Winner {
    pub fn new(render_ctx: &mut RenderContext) -> Winner {
        let mut background_asset: Box<dyn Asset<Texture>> =
            Box::new(Bytes(include_bytes!("../../../assets/winner.png")));

        let background = loop {
            match background_asset.poll(render_ctx).unwrap() {
                AssetState::Done(texture) => break texture,
                AssetState::Loading(new_state) => background_asset = new_state,
            }
        };

        let background_view = background.create_default_view();

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

        let transformations: [[f32; 4]; 4] = nalgebra::Matrix4::new_nonuniform_scaling(
            &nalgebra::Vector3::new(1. / 768., 1. / 576., 1.),
        )
        .append_scaling(2.)
        .append_translation(&nalgebra::Vector3::new(-1., -1., 0.))
        .into();

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
                        resource: wgpu::BindingResource::TextureView(&background_view),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &transforms_buffer,
                            range: 0..std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                        },
                    },
                ],
            });

        let quad = Quad::new(0., 0., 768., 576., 768., 576.);
        let (vertices, indices) = quad.vertices_and_indices2d([0., 0.], [1., 1., 1., 1.]);

        let vertex_buf = render_ctx
            .device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        let index_buf = render_ctx
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);

        let buffers = (vertex_buf, index_buf);

        Winner {
            background,
            background_view,
            bind_group,
            sampler,
            buffers,
        }
    }
}

impl State for Winner {
    type Wrapper = States;
    fn name(&self) -> &str {
        "Loading"
    }

    fn update(
        self,
        _delta: Duration,
        _io_state: &IoState,
        render_ctx: &mut RenderContext,
        _debug_ctx: &mut DebugContext,
    ) -> States {
        States::Win(self)
    }

    fn redraw(&self, render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) {
        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_ctx.frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::TRANSPARENT,
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&self.buffers.1, 0);
        rpass.set_vertex_buffers(0, &[(&self.buffers.0, 0)]);
        rpass.draw_indexed(0..6, 0, 0..1);
    }
}

impl DebugDrawable for Winner {}

pub struct Loser {
    assets: GameAssets,
    background: Texture,
    background_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    buffers: (wgpu::Buffer, wgpu::Buffer),
}

impl Loser {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Loser {
        let mut background_asset: Box<dyn Asset<Texture>> =
            Box::new(Bytes(include_bytes!("../../../assets/loser.png")));

        let background = loop {
            match background_asset.poll(render_ctx).unwrap() {
                AssetState::Done(texture) => break texture,
                AssetState::Loading(new_state) => background_asset = new_state,
            }
        };

        let background_view = background.create_default_view();

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

        let transformations: [[f32; 4]; 4] = nalgebra::Matrix4::new_nonuniform_scaling(
            &nalgebra::Vector3::new(1. / 768., 1. / 576., 1.),
        )
        .append_scaling(2.)
        .append_translation(&nalgebra::Vector3::new(-1., -1., 0.))
        .into();

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
                        resource: wgpu::BindingResource::TextureView(&background_view),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &transforms_buffer,
                            range: 0..std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                        },
                    },
                ],
            });

        let quad = Quad::new(0., 0., 768., 576., 768., 576.);
        let (vertices, indices) = quad.vertices_and_indices2d([0., 0.], [1., 1., 1., 1.]);

        let vertex_buf = render_ctx
            .device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        let index_buf = render_ctx
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);

        let buffers = (vertex_buf, index_buf);

        Loser {
            assets,
            background,
            background_view,
            bind_group,
            sampler,
            buffers,
        }
    }
}

impl State for Loser {
    type Wrapper = States;
    fn name(&self) -> &str {
        "Loading"
    }

    fn update(
        self,
        _delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        _debug_ctx: &mut DebugContext,
    ) -> States {
        if io_state.pressed(57) {
            let Loser { assets, .. } = self;

            States::Play(Play::new(assets, render_ctx))
        } else {
            States::Lose(self)
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) {
        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_ctx.frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::TRANSPARENT,
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&self.buffers.1, 0);
        rpass.set_vertex_buffers(0, &[(&self.buffers.0, 0)]);
        rpass.draw_indexed(0..6, 0, 0..1);
    }
}

impl DebugDrawable for Loser {}
