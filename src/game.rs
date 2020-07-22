//! here we go again

use std::collections::HashMap;
use std::time::Duration;

use winit::{event::Event, event_loop::ControlFlow};

use crate::debug::DebugContext;
use crate::gfx::RenderContext;

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
    pub keys: HashMap<winit::event::ScanCode, bool>,
    pub last_frame_keys: HashMap<winit::event::ScanCode, bool>,
}

impl IoState {
    pub fn set_key(&mut self, key: winit::event::ScanCode, state: bool) {
        self.keys.insert(key, state);
    }

    pub fn is_down(&self, key: winit::event::ScanCode) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }

    pub fn pressed(&self, key: winit::event::ScanCode) -> bool {
        self.last_frame_keys
            .get(&key)
            .and_then(|last_state| {
                let cur_state = self.keys.get(&key).unwrap_or(&false);
                Some(*last_state == false && *cur_state == true)
            })
            .unwrap_or(false)
    }

    pub fn update(&mut self) {
        self.last_frame_keys = self.keys.clone();
    }
}
