//! Default 3D shader with depth buffer.

use crate::gfx::pass::Pass;
use crate::gfx::primitives::MVP;
use crate::gfx::{default_flat_pipeline, Mesh, RenderContext, Scene};
use std::rc::Rc;
use wgpu::BindGroup;

pub struct Default {
    global_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Default {
    pub fn new(render_ctx: &mut RenderContext) -> Default {
        let (pipeline, global_bind_group_layout, model_bind_group_layout) =
            default_flat_pipeline(render_ctx.device);

        Default {
            pipeline,
            global_bind_group_layout,
            model_bind_group_layout,
        }
    }

    fn global_bind_group<S: Scene>(
        &self,
        scene: &S,
        render_ctx: &mut RenderContext,
    ) -> wgpu::BindGroup {
        let camera = scene.camera();
        // TODO: get rid of zerobytes
        use zerocopy::AsBytes;

        let transforms_buffer = render_ctx
            .device
            .create_buffer_with_data(&camera.as_bytes(), wgpu::BufferUsage::UNIFORM);

        render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.global_bind_group_layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_buffer,
                        range: 0..std::mem::size_of::<MVP>() as u64,
                    },
                }],
                label: None,
            })
    }

    fn model_bind_group(&self, mesh: &dyn Mesh, render_ctx: &mut RenderContext) -> wgpu::BindGroup {
        // TODO: get rid of zerobytes
        use zerocopy::AsBytes;

        let transform_matrix: [[f32; 4]; 4] = mesh.transformation().into();

        let transforms_buffer = render_ctx
            .device
            .create_buffer_with_data(&transform_matrix.as_bytes(), wgpu::BufferUsage::UNIFORM);

        render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.model_bind_group_layout,
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
}

impl<S> Pass<S> for Default
where
    S: Scene,
{
    fn render(&self, scene: &S, render_ctx: &mut RenderContext) -> () {
        let global_bind_group = self.global_bind_group(scene, render_ctx);

        // collect buffers and make bind groups
        let meshes = scene.meshes();
        let (lower_bound, upper_bound) = meshes.size_hint();
        let mut mesh_render_data = Vec::with_capacity(upper_bound.unwrap_or(lower_bound));

        struct MeshData {
            index_buffer: Rc<wgpu::Buffer>,
            index_offset: u64,
            index_size: u64,
            index_count: u32,
            vertex_buffers: Vec<(Rc<wgpu::Buffer>, u64, u64)>,
            bind_group: wgpu::BindGroup,
        }

        for mesh in meshes {
            let (index_buffer, index_offset, index_size) = mesh.index_buffer();
            let index_count = (index_size / 2) as u32; // TODO: implement properly
            let vertex_buffers = vec![
                mesh.positions_buffer(),
                mesh.normals_buffer(),
                mesh.texcoords_buffer(),
            ];

            let bind_group = self.model_bind_group(mesh, render_ctx);

            mesh_render_data.push(MeshData {
                index_buffer,
                index_offset,
                index_size,
                index_count,
                vertex_buffers,
                bind_group,
            });
        }

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

        rpass.set_pipeline(&self.pipeline);

        rpass.set_bind_group(0, &global_bind_group, &[]);

        for mesh in &mesh_render_data {
            rpass.set_bind_group(1, &mesh.bind_group, &[]);

            rpass.set_index_buffer(
                mesh.index_buffer.as_ref(),
                mesh.index_offset,
                mesh.index_size,
            );

            for (i, (buf, offset, len)) in mesh.vertex_buffers.iter().enumerate() {
                rpass.set_vertex_buffer(i as u32, buf.as_ref(), *offset, *len);
            }

            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
