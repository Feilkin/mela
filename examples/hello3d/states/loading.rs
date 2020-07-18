use crate::states::{Play, States};
use image::load;
use mela::asset::scene::Scene;
use mela::asset::{Asset, AssetState};
use mela::debug::{DebugContext, DebugDrawable};
use mela::game::IoState;
use mela::gfx::{RenderContext, Texture};
use mela::state::State;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

pub struct RequiredGameAssets {
    scene: Box<dyn Asset<Scene>>,
}

pub struct GameAssets {
    pub scene: Scene,
}

pub struct Loading {
    remaining: RequiredGameAssets,
}

impl Loading {
    pub fn new() -> Loading {
        Loading {
            remaining: RequiredGameAssets {
                scene: Box::new("assets/models/minigolf_course_test.gltf"),
            },
        }
    }
}

impl State for Loading {
    type Wrapper = States;

    fn name(&self) -> &str {
        "Loading"
    }

    fn update(
        self,
        _delta: Duration,
        _io_state: &IoState,
        render_ctx: &mut RenderContext,
        _debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        let Loading { remaining } = self;

        let mut remaining_count = 0;

        match remaining.scene.poll(render_ctx).unwrap() {
            AssetState::Done(scene) => {
                let loaded = GameAssets { scene };

                States::Play(Play::new(loaded, render_ctx))
            }
            AssetState::Loading(new_state) => States::Loading(Loading {
                remaining: RequiredGameAssets { scene: new_state },
            }),
        }
    }

    fn redraw(&self, _render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) {
        unimplemented!()
    }
}

impl DebugDrawable for Loading {}
