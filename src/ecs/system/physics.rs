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
use nphysics3d::world::{GeometricalWorld, MechanicalWorld};
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::RwLock;

pub struct PhysicsWorld<T: RealField> {
    pub mechanical_world: DefaultMechanicalWorld<T>,
    pub geometrical_world: DefaultGeometricalWorld<T>,
    pub bodies: DefaultBodySet<T>,
    pub colliders: DefaultColliderSet<T>,
    pub constraints: DefaultJointConstraintSet<T>,
    pub force_generators: DefaultForceGeneratorSet<T>,
}

impl<T: RealField> PhysicsWorld<T> {
    pub fn new(gravity: Vector3<T>) -> PhysicsWorld<T> {
        PhysicsWorld {
            mechanical_world: MechanicalWorld::new(gravity),
            geometrical_world: GeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }
}

pub struct PhysicsSystem<T: RealField> {
    update_interval: Duration,
    update_timer: Duration,
    handle_lookup: HashMap<Entity, DefaultBodyHandle>,
    physics_world: Rc<RwLock<PhysicsWorld<T>>>,
}

impl<T: RealField> PhysicsSystem<T> {
    pub fn new(physics_world: Rc<RwLock<PhysicsWorld<T>>>) -> PhysicsSystem<T> {
        PhysicsSystem {
            update_interval: Duration::from_secs_f64(1. / 60.),
            update_timer: Duration::new(0, 0),
            physics_world,
            handle_lookup: Default::default(),
        }
    }
}

impl<W: World, T: RealField> System<W> for PhysicsSystem<T>
where
    W: WorldStorage<PhysicsBody<T>> + WorldStorage<Transform<T>>,
{
    type SystemData<'a> = (Write<'a, PhysicsBody<T>>, Write<'a, Transform<T>>);

    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn update<'f>(
        &mut self,
        (mut body_reader, mut transform_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        let mut physics_world_guard = self.physics_world.write().unwrap();
        let &mut PhysicsWorld {
            ref mut mechanical_world,
            ref mut geometrical_world,
            ref mut bodies,
            ref mut colliders,
            ref mut constraints,
            ref mut force_generators,
        } = physics_world_guard.deref_mut();

        self.update_timer += delta;
        if self.update_timer < self.update_interval {
            return;
        }

        self.update_timer -= self.update_interval;

        for (entity, body_desc) in body_reader.iter_mut() {
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

                let body_handle = bodies.insert(body);

                body_desc.handle = Some(body_handle);

                if body_desc.colliders.len() > 1 {
                    unimplemented!("multibodies")
                }

                // add all colliders associated with this body
                for (i, collider_desc) in body_desc.colliders.iter().enumerate() {
                    let collider = collider_desc.build(BodyPartHandle(body_handle, i));
                    colliders.insert(collider);
                }

                self.handle_lookup.insert(entity, body_handle);
            }

            // TODO: remove entity bodies
        }

        mechanical_world.step(
            geometrical_world,
            bodies,
            colliders,
            constraints,
            force_generators,
        );

        // update transformations
        for (entity, _) in body_reader.iter() {
            // we can unwrap here, since we already made sure all entities who have bodies are in
            // the lookup table, and since we have a Read on the body component storage, we can be
            // sure it didn't get altered during physics world step
            let body_handle = self.handle_lookup.get(&entity).unwrap();
            // we only support rigid bodies for now, so downcasting is OK here
            let body: &RigidBody<T> = bodies.get(*body_handle).unwrap().downcast_ref().unwrap();

            let new_transform = Transform(body.position().to_homogeneous());

            transform_reader.set(entity, new_transform);
        }
    }
}
