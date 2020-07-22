//! Physics related systems

use std::collections::HashMap;
use std::time::Duration;

use nalgebra::{Matrix4, RealField, Vector3};
use ncollide3d::shape::ShapeHandle;
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{BodyPartHandle, ColliderDesc, RigidBody, RigidBodyDesc};
use nphysics3d::{
    object::{DefaultBodyHandle, DefaultBodySet, DefaultColliderSet},
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};

use crate::ecs::component::{PhysicsBody, Transform};
use crate::ecs::system::{Read, Write};
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{ComponentStorage, Entity, ReadAccess, RwAccess, System};
use crate::game::IoState;
use crate::gfx::RenderContext;

pub struct PhysicsSystem<T: RealField> {
    update_interval: Duration,
    update_timer: Duration,
    mechanical_world: DefaultMechanicalWorld<T>,
    geometrical_world: DefaultGeometricalWorld<T>,
    bodies: DefaultBodySet<T>,
    colliders: DefaultColliderSet<T>,
    constraints: DefaultJointConstraintSet<T>,
    force_generators: DefaultForceGeneratorSet<T>,
    handle_lookup: HashMap<Entity, DefaultBodyHandle>,
    // TODO: joints, force generators
}

impl<T: RealField> PhysicsSystem<T> {
    pub fn new(gravity: Vector3<T>) -> PhysicsSystem<T> {
        let mut mechanical_world = DefaultMechanicalWorld::new(gravity);

        PhysicsSystem {
            update_interval: Duration::from_secs_f64(1. / 60.),
            update_timer: Duration::new(0, 0),
            mechanical_world,
            geometrical_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
            handle_lookup: Default::default(),
        }
    }
}

impl<W: World, T: RealField> System<W> for PhysicsSystem<T>
where
    W: WorldStorage<PhysicsBody<T>> + WorldStorage<Transform<T>>,
{
    type SystemData<'a> = (Read<'a, PhysicsBody<T>>, Write<'a, Transform<T>>);

    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn update<'f>(
        &mut self,
        (body_reader, mut transform_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        self.update_timer += delta;
        if self.update_timer < self.update_interval {
            return;
        }

        self.update_timer -= self.update_interval;

        for (entity, body_desc) in body_reader.iter() {
            if !self.handle_lookup.contains_key(&entity) {
                let transform = transform_reader
                    .fetch(entity)
                    .expect("entity missing transformation");

                // try converting transform matrix into isometry
                let position = nalgebra::try_convert_ref(&transform.0).unwrap();

                let body = RigidBodyDesc::new()
                    .mass(body_desc.mass)
                    .status(body_desc.body_status)
                    .position(position)
                    .linear_damping(body_desc.linear_damping)
                    .angular_damping(body_desc.angular_damping)
                    .build();

                let body_handle = self.bodies.insert(body);

                if body_desc.colliders.len() > 1 {
                    unimplemented!("multibodies")
                }

                // add all colliders associated with this body
                for (i, collider_desc) in body_desc.colliders.iter().enumerate() {
                    let collider = collider_desc.build(BodyPartHandle(body_handle, i));
                    self.colliders.insert(collider);
                }

                self.handle_lookup.insert(entity, body_handle);
            }

            // TODO: remove entity bodies
        }

        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.constraints,
            &mut self.force_generators,
        );

        // update transformations
        for (entity, _) in body_reader.iter() {
            // we can unwrap here, since we already made sure all entities who have bodies are in
            // the lookup table, and since we have a Read on the body component storage, we can be
            // sure it didn't get altered during physics world step
            let body_handle = self.handle_lookup.get(&entity).unwrap();
            // we only support rigid bodies for now, so downcasting is OK here
            let body: &RigidBody<T> = self
                .bodies
                .get(*body_handle)
                .unwrap()
                .downcast_ref()
                .unwrap();

            let new_transform = Transform(body.position().to_homogeneous());

            transform_reader.set(entity, new_transform);
        }
    }
}
