use crate::states::loading::GameAssets;
use crate::states::States;
use crate::world::MyWorld;
use gltf::camera::Projection;
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::system::physics::PhysicsSystem;
use mela::ecs::system::SystemCaller;
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
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let scene =
            DefaultScene::from_gltf(assets.scene.document, assets.scene.buffers, render_ctx);

        let pass = DefaultPass::new(render_ctx);
        let mut world = MyWorld::default();

        let systems =
            vec![Box::new(PhysicsSystem::new(Vector3::y() * 9.81f32))
                as Box<dyn SystemCaller<MyWorld>>];

        Play {
            scene,
            pass,
            world,
            systems,
        }
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
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        let Play {
            world, mut systems, ..
        } = self;

        for system in &mut systems {
            system.dispatch(&world, delta, io_state, render_ctx);
        }

        States::Play(Play {
            world,
            systems,
            ..self
        })
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        self.pass.render(&self.scene, render_ctx);
    }
}

impl DebugDrawable for Play {}
