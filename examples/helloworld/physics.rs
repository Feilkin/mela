use legion::maybe_changed;
use mela::components::Transform;
use mela::ecs::systems::CommandBuffer;
use mela::ecs::world::SubWorld;
use mela::ecs::{Entity, EntityStore, Query, World};
use mela::game::PhysicsStuff;
use mela::na as nalgebra;
use mela::na::Vector3;
use mela::na::{vector, Translation};
use mela::Delta;
use rapier3d::dynamics::{
    CCDSolver, IntegrationParameters, IslandManager, JointSet, RigidBody, RigidBodyBuilder,
    RigidBodySet,
};
use rapier3d::geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier3d::pipeline::PhysicsPipeline;
use rapier3d::prelude::{Collider, ColliderHandle, Joint, JointParams, RigidBodyHandle};
use std::collections::HashMap;

pub struct PhysicsBody {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub joints: Vec<(Entity, JointParams)>,
}

#[mela::ecs::system]
pub fn add_physics_handles(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    query: &mut Query<(
        Entity,
        &PhysicsBody,
        &Transform,
        Option<&RigidBodyHandle>,
        Option<&ColliderHandle>,
    )>,
    #[resource] stuff: &mut PhysicsStuff,
) {
    // look up for new body handles
    let mut new_bodies: HashMap<Entity, RigidBodyHandle> = HashMap::new();

    for (entity, body, transform, handle, _) in query.iter(world) {
        if handle.is_none() {
            let mut rigid_body: RigidBody = body.rigid_body.clone();
            rigid_body.set_translation(transform.0.translation.vector.clone(), true);

            let body_handle = stuff.rigid_body_set.insert(rigid_body);
            let collider_handle = stuff.collider_set.insert_with_parent(
                body.collider.clone(),
                body_handle.clone(),
                &mut stuff.rigid_body_set,
            );

            cmd.add_component(*entity, body_handle.clone());
            cmd.add_component(*entity, collider_handle.clone());
            new_bodies.insert(*entity, body_handle);
        }
    }

    query.for_each(
        world,
        |(entity, body, _, handle, _): (&Entity, &PhysicsBody, _, Option<_>, _)| {
            if handle.is_none() && !body.joints.is_empty() {
                for (other, joint) in &body.joints {
                    let own_body = new_bodies[&entity];
                    let other_body = new_bodies.get(other).expect("Could not find joint entity");
                    stuff.joint_set.insert(own_body, *other_body, joint.clone());
                }
            }
        },
    );
}

#[mela::ecs::system(for_each)]
#[filter(maybe_changed::<Transform>())]
pub fn positions_to_physics(
    transform: &Transform,
    handle: &RigidBodyHandle,
    #[resource] stuff: &mut PhysicsStuff,
) {
    stuff
        .rigid_body_set
        .get_mut(handle.clone())
        .unwrap()
        .set_position(transform.0.clone(), true);
}

#[mela::ecs::system]
#[profiling::function]
pub fn physics(#[resource] stuff: &mut PhysicsStuff, #[resource] delta: &Delta) {
    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, 0.0, -9.81 * 6.];

    stuff.physics_pipeline.step(
        &gravity,
        &stuff.integration_parameters,
        &mut stuff.island_manager,
        &mut stuff.broad_phase,
        &mut stuff.narrow_phase,
        &mut stuff.rigid_body_set,
        &mut stuff.collider_set,
        &mut stuff.joint_set,
        &mut stuff.ccd_solver,
        &(),
        &(),
    );
}

#[mela::ecs::system(for_each)]
pub fn positions_from_physics(
    transform: &mut Transform,
    handle: &RigidBodyHandle,
    #[resource] stuff: &PhysicsStuff,
) {
    let new_transform = stuff.rigid_body_set.get(handle.clone()).unwrap().position();
    transform.0 = new_transform.clone();
}
