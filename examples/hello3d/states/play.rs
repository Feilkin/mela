use std::time::Duration;

use nalgebra::Vector3;

use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::system::physics::{PhysicsSystem, PhysicsWorld};
use mela::ecs::system::scene::SceneSystem;
use mela::ecs::system::SystemCaller;
use mela::game::IoState;
use mela::gfx::RenderContext;
use mela::state::State;

use crate::states::loading::GameAssets;
use crate::states::States;
use crate::systems::{CameraUnclipper, InputSystem};
use crate::world::MyWorld;
use std::rc::Rc;
use std::sync::RwLock;

pub struct Play {
    paused: bool,
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

        let physics_world = Rc::new(RwLock::new(PhysicsWorld::new(Vector3::z() * -9.81_f32)));

        let systems = vec![
            Box::new(InputSystem::new(Rc::clone(&physics_world))) as Box<dyn SystemCaller<MyWorld>>,
            Box::new(PhysicsSystem::new(Rc::clone(&physics_world)))
                as Box<dyn SystemCaller<MyWorld>>,
            Box::new(CameraUnclipper::new(Rc::clone(&physics_world)))
                as Box<dyn SystemCaller<MyWorld>>,
            Box::new(scene_system) as Box<dyn SystemCaller<MyWorld>>,
        ];

        Play {
            world,
            systems,
            paused: true,
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
        if io_state.pressed(0x19) {
            render_ctx.window.set_cursor_visible(!self.paused);
            return States::Play(Play {
                paused: !self.paused,
                ..self
            });
        }

        if self.paused {
            return States::Play(self);
        }

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
