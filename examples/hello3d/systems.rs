//! Example specific systems

use crate::world::MyWorld;
use mela::ecs::component::OrbitCamera;
use mela::ecs::system::Write;
use mela::ecs::System;
use mela::game::IoState;
use mela::gfx::RenderContext;
use nalgebra::{Rotation3, Vector3};
use std::time::Duration;

pub struct InputSystem;

impl System<MyWorld> for InputSystem {
    type SystemData<'a> = (Write<'a, OrbitCamera>,);

    fn name(&self) -> &'static str {
        "InputSystem"
    }

    fn update<'f>(
        &mut self,
        (mut camera_writer,): Self::SystemData<'f>,
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
    }
}
