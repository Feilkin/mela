use std::time::Duration;

use gltf::camera::Projection;
use nalgebra::{Matrix4, Vector3};

use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::system::physics::PhysicsSystem;
use mela::ecs::system::scene::SceneSystem;
use mela::ecs::system::SystemCaller;
use mela::ecs::world::World;
use mela::game::IoState;
use mela::gfx::pass::Pass;
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::{pass::DefaultPass, DefaultMesh, DefaultScene, RenderContext};
use mela::state::State;

use crate::states::loading::GameAssets;
use crate::states::States;
use crate::world::MyWorld;

pub struct Play {
    world: MyWorld,
    systems: Vec<Box<dyn SystemCaller<MyWorld>>>,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let mut world = MyWorld::default();

        let (scene_system, new_world) = SceneSystem::from_gltf(
            assets.scene.document,
            assets.scene.buffers,
            world,
            render_ctx,
        );
        world = new_world;

        let systems = vec![
            Box::new(PhysicsSystem::new(Vector3::z() * -9.81_f32))
                as Box<dyn SystemCaller<MyWorld>>,
            Box::new(scene_system) as Box<dyn SystemCaller<MyWorld>>,
        ];

        Play { world, systems }
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
        for system in &self.systems {
            system.render(render_ctx);
        }
    }
}

impl DebugDrawable for Play {}
