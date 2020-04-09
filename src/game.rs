//! here we go again

use std::time::Duration;

use winit::{event::Event, event_loop::ControlFlow};

use crate::debug::DebugContext;
use crate::gfx::RenderContext;
use crate::profiler;

pub trait Playable: Sized {
    /// Advances this game to next state
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self;

    /// Handle window events
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;

    /// Renders this game
    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> ();
}

//TODO: Fix this
#[derive(Default)]
pub struct IoState {
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
}
