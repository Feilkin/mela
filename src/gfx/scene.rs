//! Scene stuff

use crate::gfx::mesh::DefaultMesh;
use crate::gfx::primitives::MVP;
use crate::gfx::{Mesh, RenderContext};
use gltf::buffer::Source;
use gltf::camera::Projection;
use std::rc::Rc;

pub trait Scene {
    type MeshIter<'a>: Iterator<Item = &'a dyn Mesh>;

    fn camera(&self) -> MVP;
    fn meshes<'a, 's: 'a>(&'s self) -> Self::MeshIter<'a>;
}

pub struct DefaultScene {
    buffers: Vec<Rc<wgpu::Buffer>>,
    meshes: Vec<DefaultMesh>,
    camera: MVP,
}

impl Scene for DefaultScene {
    type MeshIter<'a> = impl Iterator<Item = &'a dyn Mesh>;

    fn camera(&self) -> MVP {
        self.camera.clone()
    }

    fn meshes<'a, 's: 'a>(&'s self) -> Self::MeshIter<'a> {
        self.meshes.iter().map(|mesh| mesh as &dyn Mesh)
    }
}

impl DefaultScene {
    pub fn from_gltf(
        document: gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        render_ctx: &mut RenderContext,
    ) -> DefaultScene {
        // upload buffers to GPU
        let buffers: Vec<_> = buffers
            .into_iter()
            .map(|b| {
                Rc::new(render_ctx.device.create_buffer_with_data(
                    &b,
                    wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::INDEX,
                ))
            })
            .collect();

        // setup camera
        let camera = document.cameras().next().expect("no camera in scene");

        let camera_node = document
            .nodes()
            .find(|n| n.name() == camera.name())
            .expect("camera defined but no node??");

        let projection = match camera.projection() {
            Projection::Perspective(ref p) => nalgebra::Matrix4::new_perspective(
                p.aspect_ratio().unwrap_or(16. / 9.),
                p.yfov(),
                p.znear(),
                p.zfar().unwrap_or(100.),
            ),
            Projection::Orthographic(ref o) => {
                nalgebra::Matrix4::new_orthographic(0., o.xmag(), 0., o.ymag(), o.znear(), o.zfar())
            }
        };

        let view: nalgebra::Matrix4<f32> = camera_node.transform().matrix().into();
        let view = nalgebra::Matrix4::look_at_rh(
            &camera_node.transform().decomposed().0.into(),
            &nalgebra::Point3::new(0., 0., 0.),
            &nalgebra::Vector3::z(),
        );

        let camera = MVP {
            view: view.into(),
            proj: projection.into(),
        };

        // setup meshes
        let scene = document
            .default_scene()
            .or(document.scenes().next())
            .expect("no scenes");
        let mut meshes = Vec::new();

        for node in document.default_scene().unwrap().nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    meshes.push(DefaultMesh::from_gltf(
                        &node, primitive, render_ctx, &buffers,
                    ));
                }
            }
        }

        DefaultScene {
            camera,
            meshes,
            buffers,
        }
    }
}
