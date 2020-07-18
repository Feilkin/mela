//! Mesh interface things

use crate::gfx::primitives::Vertex;
use crate::gfx::RenderContext;
use gltf::Semantic;
use nalgebra::Matrix4;
use std::rc::Rc;
use wgpu::Buffer;

pub trait Mesh {
    fn positions_buffer(&self) -> (Rc<wgpu::Buffer>, u64, u64);
    fn normals_buffer(&self) -> (Rc<wgpu::Buffer>, u64, u64);
    fn texcoords_buffer(&self) -> (Rc<wgpu::Buffer>, u64, u64);
    fn index_buffer(&self) -> (Rc<wgpu::Buffer>, u64, u64);
    fn transformation(&self) -> Matrix4<f32>;
}

pub struct DefaultMesh {
    positions_buffer: (Rc<wgpu::Buffer>, u64, u64),
    normals_buffer: (Rc<wgpu::Buffer>, u64, u64),
    texcoords_buffer: (Rc<wgpu::Buffer>, u64, u64),
    index_buffer: (Rc<wgpu::Buffer>, u64, u64),
    transformation: nalgebra::Matrix4<f32>,
}

impl Mesh for DefaultMesh {
    fn positions_buffer(&self) -> (Rc<Buffer>, u64, u64) {
        self.positions_buffer.clone()
    }

    fn normals_buffer(&self) -> (Rc<Buffer>, u64, u64) {
        self.normals_buffer.clone()
    }

    fn texcoords_buffer(&self) -> (Rc<Buffer>, u64, u64) {
        self.texcoords_buffer.clone()
    }

    fn index_buffer(&self) -> (Rc<Buffer>, u64, u64) {
        self.index_buffer.clone()
    }

    fn transformation(&self) -> Matrix4<f32> {
        self.transformation.clone()
    }
}

impl DefaultMesh {
    pub fn from_gltf(
        node: &gltf::Node,
        primitive: gltf::mesh::Primitive,
        render_ctx: &mut RenderContext,
        buffers: &[Rc<wgpu::Buffer>],
    ) -> DefaultMesh {
        // TODO: get rid of zerocopy
        use zerocopy::AsBytes;

        let transformation: nalgebra::Matrix4<f32> = node.transform().matrix().into();

        let positions_buffer = primitive
            .attributes()
            .find(|(semantic, _)| match semantic {
                Semantic::Positions => true,
                _ => false,
            })
            .map(|(_, accessor)| {
                let view = accessor.view().unwrap();
                let slice_offset = view.offset() as u64;
                let slice_len = view.length() as u64;
                let buffer = Rc::clone(&buffers[view.buffer().index()]);

                (buffer, slice_offset, slice_len)
            })
            .expect("no position buffer found for mesh!");

        let normals_buffer = primitive
            .attributes()
            .find(|(semantic, _)| match semantic {
                Semantic::Normals => true,
                _ => false,
            })
            .map(|(_, accessor)| {
                let view = accessor.view().unwrap();
                let slice_offset = view.offset() as u64;
                let slice_len = view.length() as u64;
                let buffer = Rc::clone(&buffers[view.buffer().index()]);

                (buffer, slice_offset, slice_len)
            })
            .expect("no normal buffer found for mesh!");

        let texcoords_buffer = primitive
            .attributes()
            .find(|(semantic, _)| match semantic {
                Semantic::TexCoords(_) => true,
                _ => false,
            })
            .map(|(_, accessor)| {
                let view = accessor.view().unwrap();
                let slice_offset = view.offset() as u64;
                let slice_len = view.length() as u64;
                let buffer = Rc::clone(&buffers[view.buffer().index()]);

                (buffer, slice_offset, slice_len)
            })
            .expect("no texcoord buffer found for mesh!");

        let indices_view = primitive.indices().unwrap().view().unwrap();

        let indices_slice_offset = indices_view.offset() as u64;
        let indices_slice_length = indices_view.length() as u64;
        let index_buffer = Rc::clone(&buffers[indices_view.buffer().index()]);

        DefaultMesh {
            transformation,
            positions_buffer,
            texcoords_buffer,
            normals_buffer,
            index_buffer: (index_buffer, indices_slice_offset, indices_slice_length),
        }
    }
}
