use std::time::Duration;

pub use loading::Loading;
use mela::debug::{DebugContext, DebugDrawable};
use mela::game::IoState;
use mela::gfx::RenderContext;
use mela::state::State;
pub use play::Play;

mod loading;
mod play;

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
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        match self {
            States::Loading(s) => s.update(delta, io_state, render_ctx, debug_ctx),
            States::Play(s) => s.update(delta, io_state, render_ctx, debug_ctx),
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
