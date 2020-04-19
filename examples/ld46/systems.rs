//! my systems :)

use crate::components::{Fire, LightC, Player, Position, Sprite};
use crate::world::MyWorld;
use mela::ecs::world::WorldStorage;
use mela::ecs::{ComponentStorage, ReadAccess, System, WriteAccess};
use mela::game::IoState;
use mela::gfx::light::Light;
use mela::gfx::primitives::Quad;
use mela::gfx::{RenderContext, Spritebatch, Texture};
use nalgebra::Point2;
use std::cmp::Ordering;
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

        // collect all sprites so we can sort them
        let mut sprites: Vec<(Sprite, nalgebra::Point2<f32>)> = components
            .sprites
            .read()
            .iter()
            .map(|(e, s)| {
                let position = components
                    .positions
                    .read()
                    .fetch(e)
                    .unwrap_or(&default_pos)
                    .0
                    .clone();

                (s.clone(), position)
            })
            .collect();

        sprites.sort_by(|a, b| a.1.coords.y.partial_cmp(&b.1.coords.y).unwrap());

        for (sprite, position) in &sprites {
            self.spritebatch
                .add_quad(&sprite.quad, position.coords.into());
            self.material_batch
                .add_quad(&sprite.quad, position.coords.into());
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
        self.spritebatch
            .draw_to(&nalgebra::Matrix4::identity(), view[0], render_ctx);
        self.material_batch
            .draw_to(&nalgebra::Matrix4::identity(), view[1], render_ctx);
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

        let (entity, player_component) = components
            .players
            .read()
            .iter()
            .next()
            .and_then(|(entity, pc)| Some((entity, pc.clone())))
            .expect("no player?");

        let position = components
            .positions
            .read()
            .fetch(entity)
            .expect("player has no position?")
            .0
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

        let direction = if movement_vector.norm() > 0. {
            movement_vector.normalize_mut();
            movement_vector.y.atan2(movement_vector.x)
        } else {
            player_component.direction
        };

        let invulnerable_timer = if let Some(iv) = player_component.invulnerable_timer {
            if iv <= delta {
                None
            } else {
                Some(iv - delta)
            }
        } else {
            None
        };

        let new_player_component = Player {
            direction,
            invulnerable_timer,
            ..player_component
        };

        components.players.write().set(entity, new_player_component);

        let mut new_pos = position + movement_vector;
        if new_pos.coords.x < 0. {
            new_pos.coords.x = 0.
        }
        if new_pos.coords.x > 768. {
            new_pos.coords.x = 768.
        }
        if new_pos.coords.y < 0. {
            new_pos.coords.y = 0.
        }
        if new_pos.coords.y > 576. {
            new_pos.coords.y = 576.
        }

        components.positions.write().set(entity, Position(new_pos));

        MyWorld {
            components,
            ..world
        }
    }
}

pub struct MoveToPlayer {}

impl System<MyWorld> for MoveToPlayer {
    fn name(&self) -> &'static str {
        "MoveToPlayer"
    }

    fn update(
        &mut self,
        delta: Duration,
        world: MyWorld,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let speed = 25.;
        let hurt_distance = 4.;
        let angry_quad = Quad::new(0., 448., 17., 27., 512., 512.);
        let idle_quad = Quad::new(32., 448., 17., 27., 512., 512.);

        let MyWorld { mut components, .. } = world;

        let (player_entity, player_component) = components
            .players
            .read()
            .iter()
            .next()
            .and_then(|(e, c)| Some((e, c.clone())))
            .expect("no player?");

        let fires_left = components.fires.read().iter().count();
        let aggro_range = if fires_left == 0 {
            10000.
        } else {
            400. - (fires_left as f32).min(6.) * 66.
        };

        let player_position = components
            .positions
            .read()
            .fetch(player_entity)
            .expect("player has no position?")
            .0
            .clone();

        for (enemy, _) in components.enemies.read().iter() {
            let enemy_position = components.positions.read().fetch(enemy).unwrap().0.clone();
            let pos_diff = &player_position - enemy_position;

            if pos_diff.norm() < hurt_distance && player_component.invulnerable_timer.is_none() {
                components.players.write().set(
                    player_entity,
                    Player {
                        health: player_component.health - 1,
                        direction: player_component.direction,
                        invulnerable_timer: Some(Duration::new(1, 0)),
                    },
                );
            }

            if pos_diff.norm() < aggro_range {
                let new_pos = &enemy_position + pos_diff.normalize() * speed * delta.as_secs_f32();
                components.positions.write().set(enemy, Position(new_pos));
                components.sprites.write().set(
                    enemy,
                    Sprite {
                        quad: angry_quad.clone(),
                    },
                );
            } else {
                components.sprites.write().set(
                    enemy,
                    Sprite {
                        quad: idle_quad.clone(),
                    },
                );
            }
        }

        MyWorld {
            components,
            ..world
        }
    }
}

pub struct LogBurner {}

impl System<MyWorld> for LogBurner {
    fn name(&self) -> &'static str {
        "LogBurner"
    }

    fn update(
        &mut self,
        delta: Duration,
        world: MyWorld,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let idle_quad = Quad::new(32., 448., 17., 27., 512., 512.);

        let MyWorld { mut components, .. } = world;

        // check that campfire is still burning
        if components.fires.read().iter().next().is_none() {
            return MyWorld {
                components,
                ..world
            };
        }

        let campfire_pos = nalgebra::Point2::new(380., 226.);
        let campfire_radius = 6.;

        let mut to_log = Vec::new();

        // TODO: manual offset bad.
        let enemy_pos_offset = nalgebra::Vector2::new(8., 16.);

        for (enemy, _) in components.enemies.read().iter() {
            let enemy_position =
                &components.positions.read().fetch(enemy).unwrap().0 + enemy_pos_offset;
            let pos_diff = &campfire_pos - enemy_position;

            if pos_diff.norm() < campfire_radius {
                to_log.push(enemy);
            }
        }

        for enemy in to_log.into_iter() {
            // remove enemy components from log
            components.enemies.write().unset(enemy);
            components.fires.write().set(
                enemy,
                Fire {
                    time_left: Duration::new(30, 0),
                },
            );
            components.sprites.write().set(
                enemy,
                Sprite {
                    quad: idle_quad.clone(),
                },
            );
        }

        MyWorld {
            components,
            ..world
        }
    }
}

pub struct FireSystem {}

impl System<MyWorld> for FireSystem {
    fn name(&self) -> &'static str {
        "FireSystem"
    }

    fn update(
        &mut self,
        delta: Duration,
        world: MyWorld,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let MyWorld {
            mut components,
            mut entities,
            ..
        } = world;

        let mut dead_fires = Vec::new();

        for (entity, fire) in components.fires.write().iter_mut() {
            if fire.time_left <= delta {
                entities
                    .iter_mut()
                    .find(|e| **e == entity)
                    .and_then(|e| {
                        *e = e.kill();
                        Some(())
                    })
                    .expect("could not find entity");
                dead_fires.push(entity);
                continue;
            }

            fire.time_left -= delta;

            let pos = components.positions.read().fetch(entity).unwrap().0.clone();
            let str = fire.strength();
            let quad = fire.quad();

            components.sprites.write().set(entity, Sprite { quad });
            components.lights.write().set(
                entity,
                LightC {
                    light: Light {
                        position: [pos.coords.x, pos.coords.y, 0.7],
                        radius: 40.0,
                        color: [1.0, 0.80, 0.20],
                        strength: str,
                        angle: 0.0,
                        sector: std::f32::consts::PI * 2.,
                        _padding: [0., 0.],
                    },
                    follow_entity: None,
                    flicker_amount: Some([1., 1.]),
                },
            )
        }

        for entity in dead_fires.into_iter() {
            components.fires.write().unset(entity);
            components.sprites.write().unset(entity);
            components.positions.write().unset(entity);
            components.lights.write().unset(entity);
        }

        MyWorld {
            components,
            entities,
            ..world
        }
    }
}

pub struct PlayerKiller {}

impl System<MyWorld> for PlayerKiller {
    fn name(&self) -> &'static str {
        "PlayerKiller"
    }

    fn update(
        &mut self,
        delta: Duration,
        world: MyWorld,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> MyWorld {
        let MyWorld {
            mut components,
            mut entities,
            ..
        } = world;

        let (entity, pc) = components
            .players
            .read()
            .iter()
            .next()
            .and_then(|(e, c)| Some((e, c.clone())))
            .expect("no player :(");

        if pc.health <= 0 {
            // player is dead :(
            components.players.write().unset(entity);
            components.sprites.write().unset(entity);
            components.positions.write().unset(entity);

            entities
                .iter_mut()
                .find(|e| **e == entity)
                .and_then(|e| {
                    *e = e.kill();
                    Some(())
                })
                .expect("could not find player??");
        }

        MyWorld {
            components,
            entities,
            ..world
        }
    }
}
