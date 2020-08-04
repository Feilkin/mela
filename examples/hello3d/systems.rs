//! Example specific systems

use crate::world::MyWorld;
use mela::ecs::component::{OrbitCamera, PhysicsBody};
use mela::ecs::system::physics::PhysicsWorld;
use mela::ecs::system::{Read, Write};
use mela::ecs::System;
use mela::game::IoState;
use mela::gfx::RenderContext;
use nalgebra::{Rotation3, Vector3};
use nphysics3d::algebra::{Force3, ForceType};
use nphysics3d::object::DefaultBodySet;
use std::rc::Rc;
use std::sync::RwLock;
use std::time::Duration;

pub struct InputSystem {
    physics_world: Rc<RwLock<PhysicsWorld<f32>>>,
}

impl InputSystem {
    pub fn new(physics_world: Rc<RwLock<PhysicsWorld<f32>>>) -> InputSystem {
        InputSystem { physics_world }
    }
}

impl System<MyWorld> for InputSystem {
    type SystemData<'a> = (Write<'a, OrbitCamera>, Read<'a, PhysicsBody<f32>>);

    fn name(&self) -> &'static str {
        "InputSystem"
    }

    fn update<'f>(
        &mut self,
        (mut camera_writer, body_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        // move camera
        let rotation_speed = std::f32::consts::PI * delta.as_secs_f32();
        let (entity, camera) = camera_writer.iter_mut().next().expect("no camera");

        if io_state.is_down(0x1e) {
            let new_rotation = &camera.rotation * Rotation3::new(Vector3::z() * -rotation_speed);
            camera.set_rotation(new_rotation);
        } else if io_state.is_down(0x20) {
            let new_rotation = &camera.rotation * Rotation3::new(Vector3::z() * rotation_speed);
            camera.set_rotation(new_rotation);
        }

        // hit ball
        if io_state.pressed(0x39) {
            let body_component = body_reader.fetch(entity).unwrap();
            if let Some(body_handle) = body_component.handle {
                println!("pushing ball");
                let (_, _, z_angle) = camera.rotation.euler_angles();
                let direction =
                    Rotation3::new(Vector3::z() * z_angle).transform_vector(&Vector3::y());
                let mut physics_world = self.physics_world.write().unwrap();
                let body = physics_world.bodies.get_mut(body_handle).unwrap();
                body.apply_force(
                    0,
                    &Force3::new(direction * 0.2, nalgebra::zero()),
                    ForceType::Impulse,
                    true,
                );
            }
        }
    }
}
