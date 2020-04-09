//! Debugging utilities

use crate::gfx::RenderContext;
use crate::profiler::OpenFrame;

pub struct DebugContext {
    //    pub profiler_frame: OpenFrame,
}

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, render_ctx: &mut RenderContext) {}
}
