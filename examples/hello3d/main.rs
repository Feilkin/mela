//! 3D graphics demo

use mela::application::Application;
use mela::debug::DebugContext;
use mela::game::Playable;
use mela::gfx::RenderContext;
use std::time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

struct Hello3dGame {}

impl Hello3dGame {
    pub fn new() -> Hello3dGame {
        Hello3dGame {}
    }
}

impl Playable for Hello3dGame {
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self {
        self
    }

    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow> {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => Some(ControlFlow::Exit),
            _ => None,
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> () {
        //        let mut rpass = render_ctx
        //            .encoder
        //            .begin_render_pass(&wgpu::RenderPassDescriptor {
        //                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
        //                    attachment: &render_ctx.frame.view,
        //                    resolve_target: None,
        //                    load_op: wgpu::LoadOp::Clear,
        //                    store_op: wgpu::StoreOp::Store,
        //                    clear_color: wgpu::Color::GREEN,
        //                }],
        //                depth_stencil_attachment: None,
        //            });
        //
        //        rpass.set_pipeline(&render_ctx.default_pipeline);
        //        rpass.set_bind_group(0, &render_ctx.default_bindgroup, &[]);
        //        rpass.draw(0..3, 0..1);
    }
}

pub fn main() {
    let game = Hello3dGame::new();
    let app = Application::new(game, "Hello 3D");

    app.run();
}
