//! Default ECS components

use crate::ecs::system::{Read, SystemData};
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage};
use crate::gfx::Mesh;
use nalgebra::{Matrix4, RealField, UnitQuaternion, Vector3};
use ncollide3d::shape::ShapeHandle;
use serde::export::Formatter;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Transform<T: RealField>(pub Matrix4<T>);

impl<T: RealField> Deref for Transform<T> {
    type Target = Matrix4<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: RealField> Component for Transform<T> {}

impl<'a, W: World, T: RealField> SystemData<'a, W> for Read<'a, Transform<T>>
where
    W: WorldStorage<Transform<T>>,
{
    fn get(world: &'a W) -> Read<'a, Transform<T>> {
        Read::new(Box::new(world.storage().read()))
    }
}

pub struct PhysicsBody<T: RealField> {
    pub shapes: Vec<(ShapeHandle<T>, T)>,
    pub mass: T,
}

impl<T: RealField> std::fmt::Debug for PhysicsBody<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<T: RealField> Component for PhysicsBody<T> {}

impl<'a, W: World, T: RealField> SystemData<'a, W> for Read<'a, PhysicsBody<T>>
where
    W: WorldStorage<PhysicsBody<T>>,
{
    fn get(world: &'a W) -> Read<'a, PhysicsBody<T>> {
        Read::new(Box::new(world.storage().read()))
    }
}

pub struct MeshComponent<M: Mesh + Send + Sync> {
    pub primitives: Vec<Arc<M>>,
}

impl<M: Mesh + Send + Sync> std::fmt::Debug for MeshComponent<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<M: Mesh + Send + Sync> Component for MeshComponent<M> {}

impl<'a, W: World, M: 'a + Mesh + Send + Sync> SystemData<'a, W> for Read<'a, MeshComponent<M>>
where
    W: WorldStorage<MeshComponent<M>>,
{
    fn get(world: &'a W) -> Read<'a, MeshComponent<M>> {
        Read::new(Box::new(world.storage().read()))
    }
}
