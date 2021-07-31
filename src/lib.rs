//! My game framework
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// re-export used libraries?
// do we want to wrap this instead? probably not
pub use imgui;
pub use itertools;
pub use lyon;
pub use nalgebra;
#[cfg(feature = "2d")]
pub use nphysics2d as nphysics;
#[cfg(feature = "3d")]
pub use nphysics3d as nphysics;
pub use winit;

pub mod application;
pub mod debug;
pub mod ecs;
pub mod game;
pub mod gfx;
pub mod scene;

pub use application::Application;
pub use game::SceneGame;
