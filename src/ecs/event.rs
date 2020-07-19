//! ECS events, eq EntityAdded, EntityRemoved

use crate::ecs::Entity;

pub enum Event {
    EntityAdded(Entity),
    EntityRemoved(Entity),
}
