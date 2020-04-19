//! my systems :)

use crate::components::{Position, Sprite};
use crate::world::MyWorld;
use mela::ecs::world::WorldStorage;
use mela::ecs::{ComponentStorage, ReadAccess, System, WriteAccess};
use mela::game::IoState;
use mela::gfx::{RenderContext, Spritebatch, Texture};
use mela::profiler::OpenTagTree;
use nalgebra::Point2;
use std::time::Duration;

pub struct SpriteSystem {
    spritebatch: Spritebatch,
    material_batch: Spritebatch,
}

impl SpriteSystem {
    pub fn new(texture: Texture, material: Texture) -> SpriteSystem {
        SpriteSystem {
            spritebatch: Spritebatch::new(texture),
            material_batch: Spritebatch::new(material),
        }
    }
}

impl System<MyWorld> for SpriteSystem {
    fn name(&self) -> &'static str {
        "SpriteSystem"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        _io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let MyWorld { components, .. } = world;

        self.spritebatch.clear();
        self.material_batch.clear();

        let default_pos = Position(Point2::new(0., 0.));

        for (entity, sprite) in components.sprites.read().iter() {
            let position = components
                .positions
                .read()
                .fetch(entity)
                .unwrap_or(&default_pos)
                .clone();

            self.spritebatch
                .add_quad(&sprite.quad, position.0.coords.into());
            self.material_batch
                .add_quad(&sprite.quad, position.0.coords.into());
        }

        self.spritebatch.update(render_ctx);
        self.material_batch.update(render_ctx);

        MyWorld {
            components,
            ..world
        }
    }

    fn draw(&self, render_ctx: &mut RenderContext) {
        self.spritebatch
            .draw(&nalgebra::Matrix4::identity(), render_ctx);
    }

    fn draw_to(&self, view: &[&wgpu::TextureView], render_ctx: &mut RenderContext) {
        self.spritebatch.draw_to(&nalgebra::Matrix4::identity(), view[0], render_ctx);
        self.material_batch.draw_to(&nalgebra::Matrix4::identity(), view[1], render_ctx);
    }
}

pub struct PlayerMvmtSystem {}

impl System<MyWorld> for PlayerMvmtSystem {
    fn name(&self) -> &'static str {
        "PlayerMvmtSystem"
    }

    fn update<'f>(
        &mut self,
        delta: Duration,
        world: MyWorld,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let speed = (50., 50.);

        let MyWorld { mut components, .. } = world;

        let (entity, _) = components.players.read().iter().next().expect("no player?");

        let position = components
            .positions
            .read()
            .fetch(entity)
            .expect("player has no position?")
            .clone();

        let mut movement_vector = nalgebra::Vector2::new(0., 0.);

        if io_state.is_down(32) {
            movement_vector.x = speed.0 * delta.as_secs_f32();
        } else if io_state.is_down(30) {
            movement_vector.x = -speed.0 * delta.as_secs_f32();
        }
        if io_state.is_down(31) {
            movement_vector.y = speed.1 * delta.as_secs_f32();
        } else if io_state.is_down(17) {
            movement_vector.y = -speed.1 * delta.as_secs_f32();
        }

        if movement_vector.norm() > 0. {
            movement_vector.normalize_mut();
        }

        components
            .positions
            .write()
            .set(entity, Position(position.0 + movement_vector));

        MyWorld {
            components,
            ..world
        }
    }
}
