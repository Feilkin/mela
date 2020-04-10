//! 3D graphics demo

use crate::states::States;
use mela::application::Application;
use mela::debug::DebugContext;
use mela::game::Playable;
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod states;

struct Hello3dGame {
    state: States,
}

impl Hello3dGame {
    pub fn new() -> Hello3dGame {
        Hello3dGame {
            state: States::new(),
        }
    }
}

impl Playable for Hello3dGame {
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self {
        let Hello3dGame { state } = self;

        let new_state = state.update(delta, render_ctx, debug_ctx);

        Hello3dGame { state: new_state }
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
        self.state.redraw(render_ctx, debug_ctx);
    }
}

pub fn main() {
    let game = Hello3dGame::new();
    let app = Application::new(game, "Hello 3D");

    app.run();
}
