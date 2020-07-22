//! Standard rendering passes.
// I have no idea what I am doing

// re-exports
pub use default::DefaultPass;
pub use pbr::Pbr;
pub use shadow::ShadowPass;

use crate::gfx::{RenderContext, Scene};

mod default;
mod pbr;
mod shadow;

pub trait Pass<S: Scene> {
    fn render(&self, scene: &S, render_ctx: &mut RenderContext) -> ();
}
