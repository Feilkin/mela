//! my components :)

use mela::ecs::world::WorldStorage;
use mela::ecs::{Component, VecStorage};
use mela::gfx::primitives::Quad;

#[derive(Default)]
pub struct Ld46Components {
    pub sprites: VecStorage<Sprite>,
    pub positions: VecStorage<Position>,
    pub players: VecStorage<Player>,
}

#[derive(Debug)]
pub struct Sprite {
    pub quad: Quad,
}

impl Component for Sprite {}

#[derive(Debug, Clone)]
pub struct Position(pub nalgebra::Point2<f32>);

impl Component for Position {}

#[derive(Debug, Clone)]
pub struct Player(pub usize);

impl Component for Player {}
