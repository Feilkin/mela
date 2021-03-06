mod loading;
mod play;

pub use loading::Loading;
pub use play::Play;

use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;

pub enum States {
    Loading(Loading),
    Play(Play),
}

impl States {
    pub fn new() -> States {
        States::Loading(Loading::new())
    }
}

impl State for States {
    type Wrapper = States;

    fn name(&self) -> &str {
        match self {
            States::Loading(s) => s.name(),
            States::Play(s) => s.name(),
        }
    }

    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        match self {
            States::Loading(s) => s.update(delta, render_ctx, debug_ctx),
            States::Play(s) => s.update(delta, render_ctx, debug_ctx),
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        match self {
            States::Loading(s) => s.redraw(render_ctx, debug_ctx),
            States::Play(s) => s.redraw(render_ctx, debug_ctx),
        }
    }
}

impl DebugDrawable for States {}
