//! World is the container thing

use crate::ecs::{
    entity::EntityBuilder, Component, ComponentStorage, Entity, ReadAccess, WriteAccess,
};

pub trait World: Sized {
    fn entities(&self) -> &[Entity];
    fn add_entity(self) -> EntityBuilder<Self>;
    fn remove_entity(self, entity: Entity) -> Self;
    fn remove_dead(self) -> Self;
}

pub trait WorldStorage<C: Component>: World {
    type Storage: ComponentStorage<C>;

    fn storage(&self) -> &Self::Storage;
    fn mut_storage(&mut self) -> &mut Self::Storage;
}
