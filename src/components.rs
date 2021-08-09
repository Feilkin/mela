//! Core ECS components

use crate::nalgebra::Isometry3;

pub struct Transform(pub Isometry3<f32>);
