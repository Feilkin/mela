//! Entity related stuff

use std::ops::Deref;

use serde::export::fmt::Error;
use serde::export::{Formatter, PhantomData};

use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage, WriteAccess};

/// Entities are just very complicated 64 bit numbers
/// First 52 bits are the unique identity of this entity.
/// Last bit (at position 63) tells if this entity is alive or dead.
/// Rest are reserved.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u64);

impl Entity {
    pub fn new(id: u64) -> Entity {
        Entity(id)
    }

    pub fn is_dead(&self) -> bool {
        self.0 >> 63 == 1
    }

    pub fn kill(&mut self) -> Entity {
        Entity(self.0 | 1 << 63)
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]",
            if self.is_dead() { '*' } else { ' ' },
            usize::from(self)
        )
    }
}

impl From<&Entity> for usize {
    fn from(e: &Entity) -> Self {
        (e.0 & 0x00_0f_ff_ff_ff_ff_ff_ff) as usize
    }
}

impl From<Entity> for usize {
    fn from(e: Entity) -> Self {
        (e.0 & 0x00_0f_ff_ff_ff_ff_ff_ff) as usize
    }
}

impl From<usize> for Entity {
    fn from(i: usize) -> Self {
        Entity(i as u64)
    }
}

/// Adds new entity, and possible components, to a World.
pub struct EntityBuilder<W: World> {
    new_entity: Entity,
    world: W,
}

impl<W: World> EntityBuilder<W> {
    /// constructs a new EntityBuilder from new entity, and a World.
    /// Consumes the World.
    pub fn new(new_entity: Entity, world: W) -> EntityBuilder<W> {
        EntityBuilder { new_entity, world }
    }

    /// consumes this entity builder, returning the new world
    pub fn build(self) -> W {
        self.world
    }

    /// adds Component for the new Entity into the World
    pub fn with_component<T>(self, component: T) -> EntityBuilder<W>
    where
        T: Component,
        W: WorldStorage<T>,
    {
        let EntityBuilder {
            new_entity,
            mut world,
            ..
        } = self;

        world.storage().write().set(new_entity, component);

        EntityBuilder {
            new_entity,
            world,
            ..self
        }
    }

    /// builds the entity, immediately calling add_entity on the underlying World.
    ///
    /// Allows chaining of `add_entity()` calls.
    pub fn add_entity(self) -> EntityBuilder<W> {
        self.build().add_entity()
    }
}
