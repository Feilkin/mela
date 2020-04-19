//! 3D graphics demo

use crate::states::States;
use mela::application::{Application, Settings};
use mela::debug::DebugContext;
use mela::game::{IoState, Playable};
use mela::gfx::RenderContext;
use mela::state::State;
use std::time::Duration;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::ControlFlow;

mod components;
mod states;
mod systems;
mod world;

struct Ld46Game {
    state: States,
    io_state: IoState,
}

impl Ld46Game {
    pub fn new() -> Ld46Game {
        Ld46Game {
            state: States::new(),
            io_state: IoState::default(),
        }
    }
}

impl Playable for Ld46Game {
    fn update(
        self,
        delta: Duration,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self {
        let Ld46Game {
            state,
            mut io_state,
        } = self;

        io_state.update();

        let new_state = state.update(delta, &io_state, render_ctx, debug_ctx);

        Ld46Game {
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
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                ..
            } => {
                self.io_state
                    .set_key(input.scancode, input.state == ElementState::Pressed);
                None
            }
            _ => None,
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) -> () {
        self.state.redraw(render_ctx, debug_ctx);
    }
}

pub fn main() {
    env_logger::init();

    let game = Ld46Game::new();
    let app = Application::new_with_settings(
        game,
        "Ludum Dare 46: Keep it alive",
        Settings {
            window_size: [768. * 2., 576. * 2.],
        },
    );

    app.run();
}
