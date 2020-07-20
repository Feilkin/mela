//! 3D graphics demo

use crate::states::States;
use mela::application::Application;
use mela::debug::DebugContext;
use mela::game::{IoState, Playable};
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod components;
mod states;
mod world;

struct Hello3dGame {
    state: States,
    io_state: IoState,
}

impl Hello3dGame {
    pub fn new() -> Hello3dGame {
        Hello3dGame {
            state: States::new(),
            io_state: IoState::default(),
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
        let Hello3dGame {
            state,
            mut io_state,
        } = self;

        io_state.update();

        let new_state = state.update(delta, &io_state, render_ctx, debug_ctx);

        Hello3dGame {
            state: new_state,
            io_state,
        }
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
