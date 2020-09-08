//! Default ECS components

use std::ops::Deref;
use std::sync::Arc;

use crate::nphysics::{
    math::{Isometry, Rotation},
    object::{BodyStatus, ColliderDesc, DefaultBodyHandle},
};
use nalgebra::{Matrix4, RealField};
use serde::export::Formatter;

use crate::ecs::Component;
use crate::gfx::light::DirectionalLight;
#[cfg(feature = "3d")]
use crate::gfx::Mesh;

#[derive(Clone, Debug)]
pub struct Transform<T: RealField>(pub Isometry<T>);

impl<T: RealField> Deref for Transform<T> {
    type Target = Isometry<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: RealField> Component for Transform<T> {}

pub struct PhysicsBody<T: RealField> {
    pub colliders: Vec<ColliderDesc<T>>,
    pub body_status: BodyStatus,
    pub mass: T,
    pub linear_damping: T,
    pub angular_damping: T,
    pub handle: Option<DefaultBodyHandle>,
}

impl<T: RealField> std::fmt::Debug for PhysicsBody<T> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<T: RealField> Component for PhysicsBody<T> {}

#[cfg(feature = "3d")]
pub struct MeshComponent<M: Mesh + Send + Sync> {
    pub primitives: Vec<Arc<M>>,
}

#[cfg(feature = "3d")]
impl<M: Mesh + Send + Sync> std::fmt::Debug for MeshComponent<M> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

#[cfg(feature = "3d")]
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
    pub rotation: Rotation<f32>,
    pub projection: Matrix4<f32>,
}

impl OrbitCamera {
    pub fn set_rotation(&mut self, rotation: Rotation<f32>) -> () {
        self.rotation = rotation;
    }
}

impl Component for OrbitCamera {}
