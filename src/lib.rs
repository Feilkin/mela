//! My game framework
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::time::Duration;

// re-export used libraries?
// do we want to wrap this instead? probably not
pub use imgui;
pub use itertools;
pub use legion as ecs;
pub use lyon;
pub use nalgebra;
pub use wgpu;
pub use winit;

pub use application::Application;
pub use game::SceneGame;
use std::ops::Deref;

pub mod application;
pub mod components;
pub mod debug;
pub mod game;
pub mod gfx;

/// Delta time since last update
pub struct Delta(Duration);

impl Deref for Delta {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
