use crate::states::loading::GameAssets;
use crate::states::States;
use gltf::camera::Projection;
use mela::debug::{DebugContext, DebugDrawable};
use mela::game::IoState;
use mela::gfx::pass::Pass;
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::{pass::Default as DefaultPass, DefaultScene, RenderContext};
use mela::state::State;
use nalgebra::{Matrix4, Vector3};
use std::time::Duration;

pub struct Play {
    pass: DefaultPass,
    scene: DefaultScene,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let scene =
            DefaultScene::from_gltf(assets.scene.document, assets.scene.buffers, render_ctx);

        let pass = DefaultPass::new(render_ctx);

        Play { scene, pass }
    }
}

impl State for Play {
    type Wrapper = States;

    fn name(&self) -> &str {
        "Play"
    }

    fn update(
        self,
        delta: Duration,
        _io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        States::Play(self)
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        self.pass.render(&self.scene, render_ctx);
    }
}

impl DebugDrawable for Play {}
