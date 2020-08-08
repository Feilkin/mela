//! Debugging utilities

use crate::gfx::RenderContext;

pub struct DebugContext<'ui> {
    pub ui: imgui::Ui<'ui>,
    //    pub profiler_frame: OpenFrame,
}

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, _render_ctx: &mut RenderContext) {}
}
