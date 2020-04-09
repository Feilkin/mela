//! Stateful game states manager factory

use std::time::Duration;

use crate::debug::{DebugContext, DebugDrawable};
use crate::game::IoState;
use crate::gfx::RenderContext;
use crate::profiler;
use crate::profiler::Profiler;

pub trait State: DebugDrawable {
    type Wrapper: State + Sized;

    /// Returns the name of this State.
    /// Mainly used for debugging.
    fn name(&self) -> &str;

    /// Updates this state to the next frame.
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper;

    /// Draws this state to screen
    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext);
}
