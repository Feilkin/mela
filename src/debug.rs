//! Debugging utilities

use crate::gfx::RenderContext;

pub struct DebugContext<'ui, 'ui_renderer> {
    pub ui: imgui::Ui<'ui>,
    pub ui_renderer: &'ui_renderer mut imgui_wgpu::Renderer,
    //    pub profiler_frame: OpenFrame,
}

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, _render_ctx: &mut RenderContext) {}
}
