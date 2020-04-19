use crate::states::loading::GameAssets;
use crate::states::States;
use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::RenderContext;
use mela::state::State;
use nalgebra::Vector3;
use std::time::Duration;

pub struct Play {
    assets: GameAssets,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let node = assets.scene.document.nodes().next().unwrap();
        let mesh = node.mesh().unwrap();

        let model_transform: nalgebra::Matrix4<f32> = node.transform().matrix().into();

        // fix coordinate system
        let model_transform =
            model_transform.append_nonuniform_scaling(&nalgebra::Vector3::new(1., -1., 1.));

        let primitive = mesh.primitives().next().unwrap();

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();

        for (semantic, accessor) in primitive.attributes() {
            let view = accessor.view().unwrap();
            let slice_offset = view.offset();
            let slice_len = view.length();
            let slice_range = slice_offset..slice_offset + slice_len;
            let buffer_index = view.buffer().index();
            let buffer = &assets.scene.buffers[buffer_index].0[slice_range];

            use gltf::Semantic;

            match semantic {
                Semantic::Positions => {
                    let layout = zerocopy::LayoutVerified::new_slice(buffer).unwrap();
                    positions.extend_from_slice(&layout);
                }
                Semantic::Normals => {
                    let layout = zerocopy::LayoutVerified::new_slice(buffer).unwrap();
                    normals.extend_from_slice(&layout);
                }
                Semantic::Colors(_) => {
                    let layout: Vec<[f32; 4]> = zerocopy::LayoutVerified::new_slice(buffer)
                        .unwrap()
                        .iter()
                        .map(|c: &[f32; 4]| [c[3], c[0], c[1], c[2]])
                        .collect();
                    colors.extend_from_slice(&layout);
                }
                _ => (),
            }
        }

        let mut vertices = Vec::with_capacity(positions.len());

        for i in 0..positions.len() {
            vertices.push(Vertex {
                position: positions[i],
                normal: normals[i],
                texture_coords: [0., 0.],
                color: colors[i],
            });
        }

        let vertex_buffer = render_ctx
            .device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        let indices_view = primitive.indices().unwrap().view().unwrap();

        let indices_slice_offset = indices_view.offset();
        let indices_slice_length = indices_view.length();
        let indices_slice = indices_slice_offset..indices_slice_offset + indices_slice_length;

        let indices_index = indices_view.buffer().index();
        let index_buffer = &assets.scene.buffers[indices_index];

        let index_buffer = render_ctx
            .device
            .create_buffer_mapped(indices_slice_length, wgpu::BufferUsage::INDEX)
            .fill_from_slice(&index_buffer[indices_slice]);

        // setup camera
        let projection = nalgebra::Matrix4::new_perspective(16. / 9., 3.14 / 4., 0.1f32, 100.0);

        let view = nalgebra::Matrix4::new_observer_frame(
            &nalgebra::Point3::new(0., 0., -1.),
            &nalgebra::Point3::new(0., 0., 0.),
            &nalgebra::Vector3::y(),
        );

        let transformations = MVP {
            model: model_transform.into(),
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
                layout: &render_ctx.pipelines.flat.1,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_buffer,
                        range: 0..std::mem::size_of::<MVP>() as u64,
                    },
                }],
            });

        Play {
            assets,
            bind_group,
            vertex_buffer,
            index_buffer,
        }
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
        let mut rpass = render_ctx
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_ctx.frame,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });

        rpass.set_pipeline(&render_ctx.pipelines.flat.0);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&self.index_buffer, 0);
        rpass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
        rpass.draw_indexed(0..47232 as u32, 0, 0..1);
    }
}

impl DebugDrawable for Play {}
