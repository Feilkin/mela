//! Default ECS components

use std::ops::Deref;
use std::sync::Arc;

use nalgebra::{Isometry3, Matrix4, RealField, Rotation3, UnitQuaternion, Vector3};
use ncollide3d::shape::ShapeHandle;
use nphysics3d::object::{ColliderDesc, DefaultBodyHandle};
use serde::export::Formatter;

use crate::ecs::system::{Read, SystemData, Write};
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage, RwAccess};
use crate::gfx::light::DirectionalLight;
use crate::gfx::Mesh;

#[derive(Clone, Debug)]
pub struct Transform<T: RealField>(pub Isometry3<T>);

impl<T: RealField> Deref for Transform<T> {
    type Target = Isometry3<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: RealField> Component for Transform<T> {}

pub struct PhysicsBody<T: RealField> {
    pub colliders: Vec<ColliderDesc<T>>,
    pub body_status: nphysics3d::object::BodyStatus,
    pub mass: T,
    pub linear_damping: T,
    pub angular_damping: T,
    pub handle: Option<DefaultBodyHandle>,
}

impl<T: RealField> std::fmt::Debug for PhysicsBody<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<T: RealField> Component for PhysicsBody<T> {}

pub struct MeshComponent<M: Mesh + Send + Sync> {
    pub primitives: Vec<Arc<M>>,
}

impl<M: Mesh + Send + Sync> std::fmt::Debug for MeshComponent<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<M: Mesh + Send + Sync> Component for MeshComponent<M> {}

#[derive(Debug)]
pub struct LightComponent {
    pub light: DirectionalLight,
}

impl Component for LightComponent {}

#[derive(Debug)]
pub struct OrbitCamera {
    pub distance: f32,
    pub max_distance: f32,
    pub min_distance: f32,
    pub rotation: Rotation3<f32>,
    pub projection: Matrix4<f32>,
}

impl OrbitCamera {
    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) -> () {
        self.rotation = rotation;
    }
}

impl Component for OrbitCamera {}
