//! Standard rendering passes.
// I have no idea what I am doing

pub use default::Default;
pub use pbr::Pbr;

// re-exports
use crate::gfx::{RenderContext, Scene};

mod default;
mod pbr;

pub trait Pass<S: Scene> {
    fn render(&self, scene: &S, render_ctx: &mut RenderContext) -> ();
}
