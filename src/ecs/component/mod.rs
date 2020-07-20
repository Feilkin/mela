//! Default ECS components

use crate::ecs::system::{Read, SystemData};
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage};
use nalgebra::{RealField, UnitQuaternion, Vector3};
use ncollide3d::shape::ShapeHandle;
use serde::export::Formatter;

#[derive(Clone, Debug)]
pub struct Transform<T: RealField> {
    pub translation: Vector3<T>,
    pub rotation: UnitQuaternion<T>,
    pub scale: Vector3<T>,
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
