//! Example specific systems

use crate::world::MyWorld;
use mela::ecs::component::{OrbitCamera, PhysicsBody, Transform};
use mela::ecs::system::physics::PhysicsWorld;
use mela::ecs::system::{Read, Write};
use mela::ecs::System;
use mela::game::IoState;
use mela::gfx::RenderContext;
use mela::nphysics3d::object::{Body, RigidBody};
use nalgebra::{Isometry3, Rotation3, Vector3};
use ncollide3d::pipeline::CollisionGroups;
use ncollide3d::query::Ray;
use nphysics3d::algebra::{Force3, ForceType};
use nphysics3d::object::DefaultBodySet;
use std::ops::DerefMut;
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
        let body_component = body_reader.fetch(entity).unwrap();

        if let Some(body_handle) = body_component.handle {
            let mut physics_world = self.physics_world.write().unwrap();
            let body: &mut RigidBody<f32> = physics_world
                .bodies
                .get_mut(body_handle)
                .unwrap()
                .downcast_mut()
                .unwrap();

            let (roll, pitch, yaw) = camera.rotation.euler_angles();
            let speed = body.velocity().linear.norm();
            if speed > 0.01 {
                // camera direction = ball direction
                let forward = Vector3::y();
                let target_yaw =
                    Rotation3::rotation_between(&forward, &body.velocity().linear.normalize())
                        .and_then(|r| Some(r.euler_angles().2))
                        .unwrap_or(0.);

                let factor = 1.0 - (-speed * delta.as_secs_f32()).exp();
                let difference = (target_yaw - yaw) % (std::f32::consts::PI * 2.);
                let difference = 2. * difference % (std::f32::consts::PI * 2.) - difference;

                let new_yaw = yaw + difference * factor;

                let new_rotation = Rotation3::from_euler_angles(roll, pitch, new_yaw);
                camera.set_rotation(new_rotation);
            } else {
                // player control
                let pitch_delta = if io_state.is_down(0x11) {
                    -rotation_speed
                } else if io_state.is_down(0x1f) {
                    rotation_speed
                } else {
                    0.
                };

                let yaw_delta = if io_state.is_down(0x1e) {
                    -rotation_speed
                } else if io_state.is_down(0x20) {
                    rotation_speed
                } else {
                    0.
                };

                camera.set_rotation(Rotation3::from_euler_angles(
                    roll + pitch_delta,
                    pitch,
                    yaw + yaw_delta,
                ));
            }

            // hit ball
            if io_state.pressed(0x39) {
                println!("pushing ball");
                let (_, _, yaw) = camera.rotation.euler_angles();
                let direction = Rotation3::new(Vector3::z() * yaw).transform_vector(&Vector3::y());
                body.apply_force(
                    0,
                    &Force3::new(direction * 0.136, nalgebra::zero()),
                    ForceType::Impulse,
                    true,
                );
            }
        }
    }
}

pub struct CameraUnclipper {
    physics_world: Rc<RwLock<PhysicsWorld<f32>>>,
}

impl CameraUnclipper {
    pub fn new(physics_world: Rc<RwLock<PhysicsWorld<f32>>>) -> CameraUnclipper {
        CameraUnclipper { physics_world }
    }
}

impl System<MyWorld> for CameraUnclipper {
    type SystemData<'a> = (Write<'a, OrbitCamera>, Read<'a, Transform<f32>>);

    fn name(&self) -> &'static str {
        "CameraUnclipper"
    }

    fn update<'f>(
        &mut self,
        (mut camera_writer, transform_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        let collision_group = CollisionGroups::new().with_blacklist(&[1]);

        let mut physics_world_guard = self.physics_world.write().unwrap();
        let &mut PhysicsWorld {
            ref mut geometrical_world,
            ref colliders,
            ..
        } = physics_world_guard.deref_mut();

        let (entity, camera) = camera_writer.iter_mut().next().unwrap();
        let transform = transform_reader.fetch(entity).unwrap();
        let isometry: Isometry3<f32> = nalgebra::try_convert_ref(&transform.0).unwrap();
        let direction = camera.rotation.transform_vector(&(Vector3::y() * -1.));

        let ray = Ray::new(isometry.translation.vector.into(), direction);

        let interferences = geometrical_world.interferences_with_ray(
            colliders,
            &ray,
            camera.max_distance,
            &collision_group,
        );

        let nearest_intersection = {
            let mut nearest = camera.max_distance;
            for (_, _, intersection) in interferences {
                //println!("collider: {:?}", obj.body());

                if intersection.toi < nearest {
                    nearest = intersection.toi;
                }
            }

            nearest
        };

        camera.distance = if nearest_intersection < camera.max_distance {
            nearest_intersection
        } else {
            camera.max_distance
        }
    }
}
