//! Debugging utilities

pub struct DebugContext<'a> {
    pub ui: imgui::Ui<'a>,
    pub ui_renderer: &'a mut imgui_wgpu::Renderer,
}

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, _debug_ctx: &mut DebugContext) {}
}
