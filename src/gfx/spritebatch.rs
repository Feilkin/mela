//! Efficient sprite batching

use wgpu::{BindGroup, Buffer};

use crate::gfx::primitives::{Quad, Vertex2D};
use crate::gfx::{RenderContext, Texture};
use wgpu::util::DeviceExt;

pub struct Spritebatch {
    texture: Texture,
    vertices: Vec<Vertex2D>,
    indices: Vec<u16>,
    dirty: bool,
    buffer: Option<(Buffer, Buffer)>,
    bind_group: Option<(BindGroup, Buffer)>,
}

impl Spritebatch {
    pub fn new(texture: Texture) -> Spritebatch {
        Spritebatch {
            texture,
            vertices: Vec::new(),
            indices: Vec::new(),
            dirty: false,
            buffer: None,
            bind_group: None,
        }
    }

    pub fn add_quad(&mut self, quad: &Quad, position: [f32; 2]) {
        self.add_quad_colored(quad, position, [1., 1., 1., 1.]);
    }

    pub fn add_quad_colored(&mut self, quad: &Quad, position: [f32; 2], color: [f32; 4]) {
        let (vertices, indices) = quad.vertices_and_indices2d(position, color);

        // We need to offset indices by amount of vertices already in the vector.
        let index_offset = self.vertices.len() as u16;

        self.vertices.extend_from_slice(&vertices);
        self.indices
            .extend(indices.iter().map(|i| i + index_offset));

        // Set dirty bit so buffers get updated.
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.dirty = true;
    }

    fn update_buffer(&mut self, render_ctx: &mut RenderContext) {
        // TODO: get rid of zerobytes
        use zerocopy::AsBytes;

        // Create buffers if they don't exist yet.
        // TODO: Figure out how to properly reuse buffers.
        //       For now, we recreate them each frame.
        //       See alse https://github.com/gfx-rs/wgpu-rs/issues/9

        let vertex_buf = render_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &self.vertices.as_bytes(),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index_buf = render_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &self.indices.as_bytes(),
                usage: wgpu::BufferUsage::INDEX,
            });

        self.buffer = Some((vertex_buf, index_buf));

        // Unset dirty bit so we know not to
        self.dirty = false;
    }

    fn setup_bind_group(&mut self, render_ctx: &mut RenderContext) {
        // TODO: get rid of zerobytes
        use zerocopy::AsBytes;

        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = render_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: Default::default(),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::Never),
            anisotropy_clamp: None,
        });

        let transformations: [[f32; 4]; 4] = nalgebra::Matrix4::new_nonuniform_scaling(
            &nalgebra::Vector3::new(1. / 768., 1. / 576., 1.),
        )
        .append_scaling(2.)
        .append_translation(&nalgebra::Vector3::new(-1., -1., 0.))
        .into();

        let transforms_buffer =
            render_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &transformations.as_bytes(),
                    usage: wgpu::BufferUsage::UNIFORM,
                });

        self.bind_group = Some((
            render_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &render_ctx.pipelines.pixel.1,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(
                                transforms_buffer
                                    .slice(0..std::mem::size_of::<[[f32; 4]; 4]>() as u64),
                            ),
                        },
                    ],
                }),
            transforms_buffer,
        ));
    }

    pub fn update(&mut self, render_ctx: &mut RenderContext) {
        if self.dirty {
            self.update_buffer(render_ctx);
        }

        if self.bind_group.is_none() {
            self.setup_bind_group(render_ctx);
        }
    }

    pub fn draw(&self, _transform: &nalgebra::Matrix4<f32>, render_ctx: &mut RenderContext) {
        if self.dirty {
            return;
        }

        // buffers are set here
        let (vertex_buf, index_buf) = match self.buffer {
            Some((ref vertex_buf, ref index_buf)) => (vertex_buf, index_buf),
            None => return,
        };

        // bind group is set here
        let (bind_group, _) = self.bind_group.as_ref().unwrap();
        //        let transform_data: [[f32; 4]; 4] = transform.clone().into();

        //        transform_buffer.map_write_async(0, std::mem::size_of::<[[f32; 4]; 4]>() as u64, move |buf| {
        //            buf.unwrap().data[0] = transform_data
        //        });
        //
        //        transform_buffer.unmap();

        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_ctx.frame,
                    resolve_target: None,
                    ops: Default::default(),
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_index_buffer(index_buf.slice(..));
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }

    pub fn draw_to(
        &self,
        _transform: &nalgebra::Matrix4<f32>,
        view: &wgpu::TextureView,
        render_ctx: &mut RenderContext,
    ) {
        if self.dirty {
            return;
        }

        // buffers are set here
        let (vertex_buf, index_buf) = match self.buffer {
            Some((ref vertex_buf, ref index_buf)) => (vertex_buf, index_buf),
            None => return,
        };

        // bind group is set here
        let (bind_group, _) = self.bind_group.as_ref().unwrap();
        //        let transform_data: [[f32; 4]; 4] = transform.clone().into();

        //        transform_buffer.map_write_async(0, std::mem::size_of::<[[f32; 4]; 4]>() as u64, move |buf| {
        //            buf.unwrap().data[0] = transform_data
        //        });
        //
        //        transform_buffer.unmap();

        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    ops: Default::default(),
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_index_buffer(index_buf.slice(..));
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}
