//! ECS world definition

use crate::components::MyComponents;
use mela::ecs::component::{MeshComponent, PhysicsBody, Transform};
use mela::ecs::system::physics::PhysicsSystem;
use mela::ecs::system::SystemCaller;
use mela::ecs::world::WorldStorage;
use mela::ecs::{entity::EntityBuilder, world::World, Entity, VecStorage};
use mela::game::IoState;
use mela::gfx::{DefaultMesh, RenderContext};
use nalgebra::Vector3;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct MyWorld {
    next_entity_id: usize,
    entities: Vec<Entity>,
    components: MyComponents,
}

impl World for MyWorld {
    fn entities(&self) -> &[Entity] {
        &self.entities
    }

    fn add_entity(self) -> EntityBuilder<Self> {
        let MyWorld {
            next_entity_id,
            mut entities,
            ..
        } = self;

        let new_entity = next_entity_id.into();
        entities.push(new_entity);

        EntityBuilder::new(
            new_entity,
            MyWorld {
                next_entity_id: next_entity_id + 1,
                entities,
                ..self
            },
        )
    }

    fn remove_entity(self, entity: Entity) -> Self {
        let MyWorld { mut entities, .. } = self;

        entities.retain(|e| *e != entity);

        MyWorld { entities, ..self }
    }

    fn remove_dead(self) -> Self {
        let MyWorld { mut entities, .. } = self;

        entities.retain(|e| !e.is_dead());

        MyWorld { entities, ..self }
    }
}

impl WorldStorage<PhysicsBody<f32>> for MyWorld {
    type Storage = VecStorage<PhysicsBody<f32>>;

    fn storage<'s, 'w: 's>(&'w self) -> &'s Self::Storage {
        &self.components.physics_bodies
    }
}

impl WorldStorage<Transform<f32>> for MyWorld {
    type Storage = VecStorage<Transform<f32>>;

    fn storage<'s, 'w: 's>(&'w self) -> &'s Self::Storage {
        &self.components.transformations
    }
}

impl WorldStorage<MeshComponent<DefaultMesh>> for MyWorld {
    type Storage = VecStorage<MeshComponent<DefaultMesh>>;

    fn storage<'s, 'w: 's>(&'w self) -> &'s Self::Storage {
        &self.components.meshes
    }
}
