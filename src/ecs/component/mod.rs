//! Default ECS components

use std::ops::Deref;

use nalgebra::{ RealField, Isometry2};

use crate::ecs::Component;

#[derive(Clone, Debug)]
pub struct Transform<T: RealField>(pub Isometry2<T>);

impl<T: RealField> Deref for Transform<T> {
    type Target = Isometry2<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: RealField> Component for Transform<T> {}