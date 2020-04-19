//! my components :)

use mela::ecs::world::WorldStorage;
use mela::ecs::{Component, Entity, VecStorage};
use mela::gfx::light::Light;
use mela::gfx::primitives::Quad;
use std::time::Duration;

#[derive(Default)]
pub struct Ld46Components {
    pub sprites: VecStorage<Sprite>,
    pub positions: VecStorage<Position>,
    pub players: VecStorage<Player>,
    pub enemies: VecStorage<Enemy>,
    pub lights: VecStorage<LightC>,
    pub fires: VecStorage<Fire>,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub quad: Quad,
}

impl Component for Sprite {}

#[derive(Debug, Clone)]
pub struct Position(pub nalgebra::Point2<f32>);

impl Component for Position {}

#[derive(Debug, Clone)]
pub struct Player {
    pub health: i32,
    pub direction: f32,
    pub invulnerable_timer: Option<Duration>,
}

impl Component for Player {}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub health: i32,
}

impl Component for Enemy {}

#[derive(Debug, Clone)]
pub struct LightC {
    pub light: Light,
    pub follow_entity: Option<Entity>,
    pub flicker_amount: Option<[f32; 2]>,
}

impl Component for LightC {}

#[derive(Debug, Clone)]
pub struct Fire {
    pub time_left: Duration,
}

impl Fire {
    pub fn strength(&self) -> f32 {
        if self.time_left > Duration::new(60, 0) {
            return 1.;
        }
        if self.time_left > Duration::new(30, 0) {
            return 0.8;
        }
        if self.time_left > Duration::new(10, 0) {
            return 0.6;
        }

        0.3
    }

    pub fn quad(&self) -> Quad {
        let str = self.strength();

        if str >= 1. {
            return Quad::new(208., 16., 16., 16., 512., 512.);
        }
        if str >= 0.8 {
            return Quad::new(224., 16., 16., 16., 512., 512.);
        }
        if str >= 0.6 {
            return Quad::new(240., 16., 16., 16., 512., 512.);
        }

        return Quad::new(256., 16., 16., 16., 512., 512.);
    }
}

impl Component for Fire {}
