use crate::states::{Play, States};
use image::load;
use mela::asset::{Asset, AssetState};
use mela::debug::{DebugContext, DebugDrawable};
use mela::gfx::{RenderContext, Texture};
use mela::state::State;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

pub struct RequiredGameAssets {
    textures: Vec<(&'static str, Box<dyn Asset<Texture>>)>,
}

#[derive(Default)]
pub struct GameAssets {
    pub textures: HashMap<String, Texture>,
}

pub struct Loading {
    remaining: RequiredGameAssets,
    loaded: GameAssets,
}

impl Loading {
    pub fn new() -> Loading {
        Loading {
            remaining: RequiredGameAssets {
                textures: vec![
                    // Spritesheet from https://www.kenney.nl/assets/bit-pack
                    (
                        "spritesheet",
                        Box::new("assets/spritesheets/1bit_colored_kenney.nl.png"),
                    ),
                ],
            },
            loaded: GameAssets::default(),
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
        render_ctx: &mut RenderContext,
        _debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        let Loading {
            remaining,
            mut loaded,
        } = self;

        let mut new_remaining = RequiredGameAssets {
            textures: Vec::with_capacity(remaining.textures.len()),
        };

        let mut remaining_count = 0;

        for (id, asset) in remaining.textures {
            match asset.poll(render_ctx).unwrap() {
                AssetState::Done(t) => {
                    loaded.textures.insert(id.to_owned(), t);
                }
                AssetState::Loading(new_state) => {
                    new_remaining.textures.push((id, new_state));
                    remaining_count += 1;
                }
            }
        }

        if remaining_count == 0 {
            States::Play(Play::new(loaded, render_ctx))
        } else {
            States::Loading(Loading {
                remaining: new_remaining,
                loaded,
            })
        }
    }

    fn redraw(&self, _render_ctx: &mut RenderContext, _debug_ctx: &mut DebugContext) {
        unimplemented!()
    }
}

impl DebugDrawable for Loading {}
