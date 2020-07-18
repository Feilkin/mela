//! Standard rendering passes.
// I have no idea what I am doing

mod default;
mod pbr;

// re-exports
use crate::gfx::{RenderContext, Scene};
pub use default::Default;
pub use pbr::Pbr;

pub trait Pass<S: Scene> {
    fn render(&self, scene: &S, render_ctx: &mut RenderContext) -> ();
}
