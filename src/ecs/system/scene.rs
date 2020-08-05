//! Scene-related systems

use std::convert::TryInto;
use std::sync::Arc;
use std::time::Duration;

use gltf::camera::Projection;
use gltf::Semantic;
use nalgebra::{
    Isometry3, Matrix4, Point3, Quaternion, Rotation3, Translation3, Unit, UnitQuaternion, Vector3,
    Vector4,
};
use ncollide3d::shape::{Ball, Compound, ShapeHandle, TriMesh};
use nphysics3d::material::{BasicMaterial, MaterialHandle};
use nphysics3d::object::{BodyStatus, ColliderDesc};
use wgpu::Buffer;

use crate::asset::scene::NodeAttributes;
use crate::ecs::component::{LightComponent, MeshComponent, OrbitCamera, PhysicsBody, Transform};
use crate::ecs::system::Read;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{ReadAccess, System};
use crate::game::IoState;
use crate::gfx::light::{DirectionalLight, LightData};
use crate::gfx::material::Materials;
use crate::gfx::pass::Pass;
use crate::gfx::primitives::MVP;
use crate::gfx::{pass::DefaultPass, DefaultMesh, DefaultScene, Mesh, RenderContext, Scene};
use ncollide3d::pipeline::CollisionGroups;

pub struct SceneSystem<M: Mesh> {
    meshes: Vec<MeshWrapper<M>>,
    buffers: Vec<Arc<wgpu::Buffer>>,
    materials: Materials,
    materials_buffer: Arc<wgpu::Buffer>,
    camera: MVP,
    lights: Vec<LightData>,
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
            + WorldStorage<PhysicsBody<f32>>
            + WorldStorage<LightComponent>
            + WorldStorage<OrbitCamera>,
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
            camera_pos: camera_node.transform().decomposed().0,
            _padding: 0.0,
        };

        // setup meshes
        let scene = document
            .default_scene()
            .or(document.scenes().next())
            .expect("no scenes");

        for node in document.default_scene().unwrap().nodes() {
            let (translation, rotation, _) = node.transform().decomposed();
            let translation_vector: Vector3<f32> = translation.into();
            let rotation_vector4: Vector4<f32> = rotation.into();
            let rotation_quaternion: Quaternion<f32> = rotation_vector4.into();
            let transform = Transform(Isometry3::from_parts(
                translation_vector.into(),
                UnitQuaternion::from_quaternion(rotation_quaternion),
            ));
            let mut entity_builder = world.add_entity().with_component(transform);

            if let Some(extras) = node.extras() {
                let attributes: NodeAttributes = serde_json::from_str(extras.get()).unwrap();

                // TODO: implement custom attributes
                if attributes.ball.unwrap_or(0) > 0 {
                    let collider_desc = ColliderDesc::new(ShapeHandle::new(Ball::new(0.010f32)))
                        .density(1.0)
                        .ccd_enabled(true)
                        .collision_groups(CollisionGroups::new().with_membership(&[0, 1]))
                        .material(MaterialHandle::new(BasicMaterial::new(0.85, 0.4)));

                    let projection = nalgebra::Matrix4::new_perspective(
                        16. / 9.,
                        0.4710899940857267,
                        0.0001,
                        100.,
                    );

                    entity_builder = entity_builder
                        .with_component(PhysicsBody {
                            body_status: BodyStatus::Dynamic,
                            colliders: vec![collider_desc],
                            mass: 0.045,
                            linear_damping: 0.8,
                            angular_damping: 0.8,
                            handle: None,
                        })
                        .with_component(OrbitCamera {
                            distance: 0.5,
                            max_distance: 1.0,
                            min_distance: 0.2,
                            rotation: Rotation3::identity(),
                            projection,
                        });
                }

                if attributes.ground.unwrap_or(0) > 0 {
                    let mesh = node.mesh().unwrap();

                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();

                    for primitive in mesh.primitives() {
                        let index_offset = vertices.len();

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

                                    indices.push(Point3::new(
                                        index_offset + x as usize,
                                        index_offset + y as usize,
                                        index_offset + z as usize,
                                    ));
                                }
                            })
                            .unwrap();
                    }

                    let mesh = TriMesh::new(vertices, indices, None);

                    entity_builder = entity_builder.with_component(PhysicsBody {
                        colliders: vec![ColliderDesc::new(ShapeHandle::new(mesh))
                            .ccd_enabled(false)
                            .collision_groups(CollisionGroups::new().with_membership(&[0]))],
                        mass: f32::INFINITY,
                        linear_damping: 0.0,
                        body_status: BodyStatus::Dynamic,
                        angular_damping: 0.0,
                        handle: None,
                    });
                }
            }

            if let Some(mesh) = node.mesh() {
                let primitives = mesh
                    .primitives()
                    .into_iter()
                    .map(|p| Arc::new(DefaultMesh::from_gltf(p, render_ctx, &gpu_buffers)))
                    .collect();
                let mesh_component = MeshComponent { primitives };

                entity_builder = entity_builder.with_component(mesh_component)
            }

            if let Some(light_desc) = node.light() {
                match light_desc.kind() {
                    gltf::khr_lights_punctual::Kind::Directional => {
                        let rotation: Unit<Quaternion<f32>> = Unit::new_normalize(
                            Quaternion::from(Vector4::from(node.transform().decomposed().1)),
                        );
                        let direction: [f32; 3] =
                            rotation.transform_vector(&Vector3::new(0., 0., -1.)).into();

                        let light = DirectionalLight::new(
                            direction,
                            light_desc.color(),
                            light_desc.intensity(),
                        );

                        entity_builder =
                            entity_builder.with_component(LightComponent { light: light })
                    }
                    _ => unimplemented!(),
                }
            }

            world = entity_builder.build();
        }

        (
            SceneSystem {
                meshes: vec![],
                lights: vec![],
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
    W: WorldStorage<MeshComponent<M>>
        + WorldStorage<Transform<f32>>
        + WorldStorage<LightComponent>
        + WorldStorage<OrbitCamera>,
{
    type SystemData<'a> = (
        Read<'a, MeshComponent<M>>,
        Read<'a, Transform<f32>>,
        Read<'a, LightComponent>,
        Read<'a, OrbitCamera>,
    );

    fn name(&self) -> &'static str {
        "SceneSystem"
    }

    fn update<'f>(
        &mut self,
        (mesh_reader, transform_reader, light_reader, camera_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        self.meshes.clear();
        self.lights.clear();

        for (entity, mesh) in mesh_reader.iter() {
            let transform = transform_reader
                .fetch(entity)
                .expect("should have transform");
            for primitive in &mesh.primitives {
                self.meshes.push(MeshWrapper {
                    mesh: Arc::clone(primitive),
                    transform: transform.to_homogeneous(),
                });
            }
        }

        for (entity, light) in light_reader.iter() {
            let transform = transform_reader
                .fetch(entity)
                .expect("should have transform");

            self.lights
                .push(light.light.light_data(&transform.to_homogeneous()));
        }

        {
            // update camera
            let (entity, camera) = camera_reader.iter().next().expect("no camera!");
            let camera_offset = camera
                .rotation
                .transform_vector(&(Vector3::y() * -camera.distance + Vector3::z() * 1.0));

            let transform = transform_reader.fetch(entity).unwrap().0.clone();
            let maybe_isometry: Option<Isometry3<f32>> = nalgebra::try_convert(transform);
            if let Some(entity_isometry) = maybe_isometry {
                let translation = camera_offset + &entity_isometry.translation.vector;
                let camera_position = translation.clone();

                let view_matrix = nalgebra::Matrix4::look_at_rh(
                    &camera_position.into(),
                    &entity_isometry.translation.vector.into(),
                    &nalgebra::Vector3::z(),
                );

                self.camera = MVP {
                    view: view_matrix.into(),
                    proj: camera.projection.clone().into(),
                    camera_pos: camera_position.into(),
                    _padding: 0.0,
                }
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

    fn lights(&self) -> &[LightData] {
        &self.lights
    }
}
