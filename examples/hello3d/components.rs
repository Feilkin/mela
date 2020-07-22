//! 3d example compnent definitions

use mela::ecs::component::{LightComponent, MeshComponent, PhysicsBody, Transform};
use mela::ecs::VecStorage;
use mela::gfx::DefaultMesh;

#[derive(Default)]
pub(crate) struct MyComponents {
    pub transformations: VecStorage<Transform<f32>>,
    pub physics_bodies: VecStorage<PhysicsBody<f32>>,
    pub meshes: VecStorage<MeshComponent<DefaultMesh>>,
    pub lights: VecStorage<LightComponent>,
}
