//! Hello, World! example mela game
//!

use crate::sdf::{SdfObject, SdfShape};
use mela::components::Transform;
use mela::na as nalgebra;
use mela::na::{point, vector, Isometry3};
use mela::{Application, Delta};

mod physics;
mod sdf;

use crate::physics::PhysicsBody;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};
use mela::debug::DebugName;
use mela::ecs::systems::CommandBuffer;
use mela::ecs::Query;
use mela::game::{IoState, PhysicsStuff};
use physics::{
    add_physics_handles_system, physics_system, positions_from_physics_system,
    positions_to_physics_system,
};
use rapier3d::dynamics::RigidBodyType;
use rapier3d::math::Rotation;
use rapier3d::na::{RealField, Vector};
use rapier3d::prelude::{
    BallJoint, ColliderBuilder, ColliderHandle, InteractionGroups, MassProperties, PrismaticJoint,
    RigidBodyBuilder, RigidBodyHandle,
};
use std::any::Any;
use std::marker::PhantomData;

#[mela::ecs::system(for_each)]
fn move_ball(
    ball_index: &BallIndex,
    body_handle: &RigidBodyHandle,
    collider_handle: &ColliderHandle,
    #[resource] io: &IoState,
    #[resource] stuff: &mut PhysicsStuff,
) {
    if ball_index.0 == 0 {
        let body = stuff.rigid_body_set.get_mut(*body_handle).unwrap();
        let collider = stuff.collider_set.get_mut(*collider_handle).unwrap();

        if io.is_down(17) {
            collider.set_friction(0.6);
            body.set_body_type(RigidBodyType::Dynamic);
            body.set_linvel(vector![10., 0., 0.], true);
        } else if io.is_down(31) {
            collider.set_friction(0.6);
            body.set_body_type(RigidBodyType::Dynamic);
            body.set_linvel(vector![-10., 0., 0.], true);
        } else {
            if body.translation().z <= 12. {
                body.set_body_type(RigidBodyType::KinematicPositionBased);
            } else {
                body.set_body_type(RigidBodyType::Dynamic);
            }
        }
    }
}

struct BallIndex(usize);

struct Animator {
    fun: Box<dyn Fn(&Entity, &mut SubWorld) -> () + Send + Sync>,
}

fn main() {
    env_logger::init();
    puffin::set_scopes_on(true);
    let mut game = mela::SceneGame::builder()
        .register_addable_component::<SdfObject>()
        .with_system(add_physics_handles_system())
        .with_system(move_ball_system())
        .with_system(positions_to_physics_system())
        .with_system(physics_system())
        .with_system(positions_from_physics_system())
        .barrier();

    let sekoilu = game
        .system_builder_for_accessing_whole_known_world("sekoilu")
        .with_query(<(Entity, &mut Animator)>::query())
        .build(|_, world, _, query| {
            let (mut animators, mut rest_of_the_world) = world.split_for_query(&query);
            for (entity, animator) in query.iter_mut(&mut animators) {
                (*animator.fun)(entity, &mut rest_of_the_world);
            }
        });

    let mut game = game
        .with_system(sekoilu)
        .barrier()
        .with_renderer::<sdf::renderer::SdfRenderer>()
        .build();

    {
        // add ground
        let rigid_body = RigidBodyBuilder::new_static().build();
        let collider = ColliderBuilder::cuboid(1000., 1000., 10.)
            .collision_groups(InteractionGroups::new(0b10, 0b11))
            .build();

        game.push_debug_entity((
            PhysicsBody {
                collider,
                rigid_body,
                joints: vec![],
            },
            DebugName("Ground cuboid".to_string()),
            Transform(Isometry3::translation(0., 0., 0.)),
            SdfObject {
                smoothing: 0.0,
                shape: SdfShape::Cuboid(1000., 1000., 1.),
            },
            Animator {
                fun: Box::new(|entity, world| {
                    let entry = world.entry_mut(*entity).unwrap();
                    let transform =
                        unsafe { entry.get_component_unchecked::<Transform>() }.unwrap();

                    transform.0.translation.z += 0.1;
                }),
            },
        ));
    }

    let rigid_body = RigidBodyBuilder::new_dynamic().build();
    let mut previous_entity = None;

    for i in (1..6).rev() {
        let radius = 2.; // + (i as f32 / 6. * f32::pi()).sin() * 3.;

        let collider = ColliderBuilder::ball(radius)
            .restitution(0.2)
            .mass_properties(MassProperties::from_ball(1., radius))
            .friction(0.1)
            .collision_groups(InteractionGroups::new(0b01, 0b10))
            .build();

        let entity = game.push_debug_entity((
            BallIndex(i),
            Transform(Isometry3::translation(-64. + i as f32 * -7., 0., 30.)),
            SdfObject {
                shape: SdfShape::Ball(radius),
                smoothing: 0.1,
            },
            PhysicsBody {
                collider,
                rigid_body: rigid_body.clone(),
                joints: if let Some(parent) = previous_entity {
                    vec![(
                        previous_entity.unwrap(),
                        {
                            let mut joint = BallJoint::new(
                                point![radius + 2., 0., 0.],
                                point![-radius - 2., 0., 0.],
                            );

                            //joint.motor_max_impulse = 300.;
                            joint
                        }
                        .into(),
                    )]
                } else {
                    vec![]
                },
            },
        ));

        previous_entity = Some(entity);
    }

    let app = Application::new(game, "Hello, World!");

    futures::executor::block_on(app.setup()).run();
}
