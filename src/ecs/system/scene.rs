//! Scene-related systems

use crate::asset::scene::NodeAttributes;
use crate::ecs::component::{MeshComponent, PhysicsBody, Transform};
use crate::ecs::system::Read;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{ReadAccess, System};
use crate::game::IoState;
use crate::gfx::material::Materials;
use crate::gfx::pass::Pass;
use crate::gfx::primitives::MVP;
use crate::gfx::{
    pass::Default as DefaultPass, DefaultMesh, DefaultScene, Mesh, RenderContext, Scene,
};
use gltf::camera::Projection;
use gltf::Semantic;
use nalgebra::{Isometry3, Matrix4, Point3, Vector3};
use ncollide3d::shape::{Ball, Compound, ShapeHandle, TriMesh};
use nphysics3d::object::{BodyStatus, ColliderDesc};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::Duration;
use wgpu::Buffer;

pub struct SceneSystem<M: Mesh> {
    meshes: Vec<MeshWrapper<M>>,
    buffers: Vec<Arc<wgpu::Buffer>>,
    materials: Materials,
    materials_buffer: Arc<wgpu::Buffer>,
    camera: MVP,
    pass: DefaultPass,
}

struct MeshWrapper<M: Mesh> {
    mesh: Arc<M>,
    transform: Matrix4<f32>,
}

impl<M: Mesh> Mesh for MeshWrapper<M> {
    fn positions_buffer(&self) -> (Arc<Buffer>, u64, u64) {
        self.mesh.positions_buffer()
    }

    fn normals_buffer(&self) -> (Arc<Buffer>, u64, u64) {
        self.mesh.normals_buffer()
    }

    fn texcoords_buffer(&self) -> (Arc<Buffer>, u64, u64) {
        self.mesh.texcoords_buffer()
    }

    fn index_buffer(&self) -> (Arc<Buffer>, u64, u64) {
        self.mesh.index_buffer()
    }

    fn transformation(&self) -> Matrix4<f32> {
        self.transform.clone()
    }

    fn material(&self) -> usize {
        self.mesh.material()
    }
}

impl<M: Mesh> From<M> for MeshWrapper<M> {
    fn from(m: M) -> Self {
        MeshWrapper {
            transform: m.transformation(),
            mesh: Arc::new(m),
        }
    }
}

impl SceneSystem<DefaultMesh> {
    pub fn from_gltf<W>(
        document: gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        mut world: W,
        render_ctx: &mut RenderContext,
    ) -> (SceneSystem<DefaultMesh>, W)
    where
        W: World
            + WorldStorage<MeshComponent<DefaultMesh>>
            + WorldStorage<Transform<f32>>
            + WorldStorage<PhysicsBody<f32>>,
    {
        let pass = DefaultPass::new(render_ctx);
        // upload buffers to GPU
        let gpu_buffers: Vec<_> = buffers
            .iter()
            .map(|b| {
                Arc::new(render_ctx.device.create_buffer_with_data(
                    &b,
                    wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::INDEX,
                ))
            })
            .collect();

        // upload materials to GPU
        let materials: Materials = document.materials().into();
        let materials_buffer = {
            use zerocopy::AsBytes;

            Arc::new(
                render_ctx
                    .device
                    .create_buffer_with_data(materials.as_bytes(), wgpu::BufferUsage::UNIFORM),
            )
        };

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

        for node in document.default_scene().unwrap().nodes() {
            let mut entity_builder = world.add_entity();

            if let Some(extras) = node.extras() {
                let attributes: NodeAttributes = serde_json::from_str(extras.get()).unwrap();

                // TODO: implement custom attributes
                if attributes.ball.unwrap_or(0) > 0 {
                    let collider_desc =
                        ColliderDesc::new(ShapeHandle::new(Ball::new(0.1f32))).density(1.0);

                    entity_builder = entity_builder.with_component(PhysicsBody {
                        body_status: BodyStatus::Dynamic,
                        colliders: vec![collider_desc],
                        mass: 1.0,
                    });
                }

                if attributes.ground.unwrap_or(0) > 0 {
                    let mesh = node.mesh().unwrap();

                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();

                    for primitive in mesh.primitives() {
                        primitive
                            .attributes()
                            .find(|(semantic, _)| match semantic {
                                Semantic::Positions => true,
                                _ => false,
                            })
                            .map(|(_, accessor)| {
                                let view = accessor.view().unwrap();
                                let slice_offset = view.offset();
                                let slice_len = view.length();
                                let buffer = &buffers[view.buffer().index()].0;
                                // FIXME: we assume positions are given as list of [f32; 3] values,
                                //        but we should support all possible types.

                                for byte_offset in
                                    (slice_offset..slice_offset + slice_len).step_by(3 * 4)
                                {
                                    let x = f32::from_le_bytes(
                                        (&buffer[byte_offset..byte_offset + 4]).try_into().unwrap(),
                                    );
                                    let y = f32::from_le_bytes(
                                        (&buffer[byte_offset + 4..byte_offset + 8])
                                            .try_into()
                                            .unwrap(),
                                    );
                                    let z = f32::from_le_bytes(
                                        (&buffer[byte_offset + 8..byte_offset + 12])
                                            .try_into()
                                            .unwrap(),
                                    );

                                    vertices.push(Point3::new(x, y, z));
                                }
                            })
                            .unwrap();
                        primitive
                            .indices()
                            .map(|accessor| {
                                let view = accessor.view().unwrap();
                                let slice_offset = view.offset();
                                let slice_len = view.length();
                                let buffer = &buffers[view.buffer().index()].0;
                                // indices are iterated in sets of 3, and collected into a vector
                                // of Point3's

                                for byte_offset in
                                    (slice_offset..slice_offset + slice_len).step_by(3 * 2)
                                {
                                    let x = u16::from_le_bytes(
                                        (&buffer[byte_offset..byte_offset + 2]).try_into().unwrap(),
                                    );
                                    let y = u16::from_le_bytes(
                                        (&buffer[byte_offset + 2..byte_offset + 4])
                                            .try_into()
                                            .unwrap(),
                                    );
                                    let z = u16::from_le_bytes(
                                        (&buffer[byte_offset + 4..byte_offset + 6])
                                            .try_into()
                                            .unwrap(),
                                    );

                                    indices.push(Point3::new(x as usize, y as usize, z as usize));
                                }
                            })
                            .unwrap();
                    }

                    let mesh = TriMesh::new(vertices, indices, None);

                    entity_builder = entity_builder.with_component(PhysicsBody {
                        colliders: vec![ColliderDesc::new(ShapeHandle::new(mesh))],
                        mass: f32::INFINITY,
                        body_status: BodyStatus::Dynamic,
                    });
                }
            }

            if let Some(mesh) = node.mesh() {
                let primitives = mesh
                    .primitives()
                    .into_iter()
                    .map(|p| Arc::new(DefaultMesh::from_gltf(p, render_ctx, &gpu_buffers)))
                    .collect();

                let transform = Transform(node.transform().matrix().into());
                let mesh_component = MeshComponent { primitives };

                entity_builder = entity_builder
                    .with_component(transform)
                    .with_component(mesh_component)
            }

            world = entity_builder.build();
        }

        (
            SceneSystem {
                meshes: vec![],
                buffers: gpu_buffers,
                materials,
                materials_buffer,
                camera,
                pass,
            },
            world,
        )
    }
}

impl<W: World, M: 'static + Mesh + Send + Sync> System<W> for SceneSystem<M>
where
    W: WorldStorage<MeshComponent<M>> + WorldStorage<Transform<f32>>,
{
    type SystemData<'a> = (Read<'a, MeshComponent<M>>, Read<'a, Transform<f32>>);

    fn name(&self) -> &'static str {
        "SceneSystem"
    }

    fn update<'f>(
        &mut self,
        (mesh_reader, transform_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        self.meshes.clear();

        for (entity, mesh) in mesh_reader.iter() {
            let transform = transform_reader
                .fetch(entity)
                .expect("should have transform");
            for primitive in &mesh.primitives {
                self.meshes.push(MeshWrapper {
                    mesh: Arc::clone(primitive),
                    transform: *transform.clone(),
                });
            }
        }
    }

    fn draw(&self, render_ctx: &mut RenderContext) {
        self.pass.render(self, render_ctx);
    }
}

impl<M: Mesh> Scene for SceneSystem<M> {
    type MeshIter<'a> = impl Iterator<Item = &'a dyn Mesh>;

    fn camera(&self) -> MVP {
        self.camera.clone()
    }

    fn meshes<'a, 's: 'a>(&'s self) -> Self::MeshIter<'a> {
        self.meshes.iter().map(|m| m as &dyn Mesh)
    }

    fn materials(&self) -> &Buffer {
        self.materials_buffer.as_ref()
    }
}
