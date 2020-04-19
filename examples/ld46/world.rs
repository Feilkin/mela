//! its my world :)

use crate::components::{Enemy, Fire, Ld46Components, LightC, Player, Position, Sprite};
use mela::ecs::entity::EntityBuilder;
use mela::ecs::world::{World, WorldStorage};
use mela::ecs::{Entity, VecStorage};

pub struct MyWorld {
    pub next_entity_id: usize,
    pub entities: Vec<Entity>,
    pub components: Ld46Components,
}

impl MyWorld {
    pub fn new() -> MyWorld {
        MyWorld {
            next_entity_id: 0,
            entities: Vec::new(),
            components: Ld46Components::default(),
        }
    }
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

impl WorldStorage<Sprite> for MyWorld {
    type Storage = VecStorage<Sprite>;

    fn storage(&self) -> &Self::Storage {
        &self.components.sprites
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.sprites
    }
}

impl WorldStorage<Position> for MyWorld {
    type Storage = VecStorage<Position>;

    fn storage(&self) -> &Self::Storage {
        &self.components.positions
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.positions
    }
}

impl WorldStorage<Player> for MyWorld {
    type Storage = VecStorage<Player>;

    fn storage(&self) -> &Self::Storage {
        &self.components.players
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.players
    }
}

impl WorldStorage<Enemy> for MyWorld {
    type Storage = VecStorage<Enemy>;

    fn storage(&self) -> &Self::Storage {
        &self.components.enemies
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.enemies
    }
}

impl WorldStorage<LightC> for MyWorld {
    type Storage = VecStorage<LightC>;

    fn storage(&self) -> &Self::Storage {
        &self.components.lights
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.lights
    }
}

impl WorldStorage<Fire> for MyWorld {
    type Storage = VecStorage<Fire>;

    fn storage(&self) -> &Self::Storage {
        &self.components.fires
    }

    fn mut_storage(&mut self) -> &mut Self::Storage {
        &mut self.components.fires
    }
}
