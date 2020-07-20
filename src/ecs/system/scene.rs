//! Scene-related systems

use crate::ecs::component::{MeshComponent, Transform};
use crate::ecs::system::Read;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{ReadAccess, System};
use crate::game::IoState;
use crate::gfx::material::Materials;
use crate::gfx::pass::Pass;
use crate::gfx::primitives::MVP;
use crate::gfx::{pass::Default as DefaultPass, DefaultScene, Mesh, RenderContext, Scene};
use nalgebra::Matrix4;
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

impl<M: Mesh + Send + Sync> SceneSystem<M> {
    pub fn from_scene<W>(
        scene: DefaultScene<M>,
        mut world: W,
        render_ctx: &mut RenderContext,
    ) -> (SceneSystem<M>, W)
    where
        W: World + WorldStorage<MeshComponent<M>> + WorldStorage<Transform<f32>>,
    {
        let DefaultScene {
            buffers,
            meshes,
            materials,
            materials_buffer,
            camera,
        } = scene;

        let pass = DefaultPass::new(render_ctx);

        for mesh in meshes {
            let transform = Transform(mesh.transformation());
            let mesh_component = MeshComponent {
                primitives: vec![Arc::new(mesh)],
            };

            world = world
                .add_entity()
                .with_component(transform)
                .with_component(mesh_component)
                .build();
        }

        (
            SceneSystem {
                meshes: vec![],
                buffers,
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
