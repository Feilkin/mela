//! 3d example compnent definitions

use mela::ecs::component::{PhysicsBody, Transform};
use mela::ecs::VecStorage;

#[derive(Default)]
pub(crate) struct MyComponents {
    pub transformations: VecStorage<Transform<f32>>,
    pub physics_bodies: VecStorage<PhysicsBody<f32>>,
}
