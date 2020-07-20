//! Physics related systems

use crate::ecs::component::{PhysicsBody, Transform};
use crate::ecs::system::Read;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{ComponentStorage, Entity, ReadAccess, RwAccess, System};
use crate::game::IoState;
use crate::gfx::RenderContext;
use nalgebra::{RealField, Vector3};
use ncollide3d::shape::ShapeHandle;
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{BodyPartHandle, ColliderDesc, RigidBody, RigidBodyDesc};
use nphysics3d::{
    object::{DefaultBodyHandle, DefaultBodySet, DefaultColliderSet},
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use std::collections::HashMap;
use std::time::Duration;

pub struct PhysicsSystem<T: RealField> {
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
        PhysicsSystem {
            mechanical_world: DefaultMechanicalWorld::new(gravity),
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
    type SystemData<'a> = (Read<'a, PhysicsBody<T>>, Read<'a, Transform<T>>);

    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn update<'f>(
        &mut self,
        (body_reader, transform_reader): Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> () {
        for (entity, body_desc) in body_reader.iter() {
            if !self.handle_lookup.contains_key(&entity) {
                let transform = transform_reader
                    .fetch(entity)
                    .expect("entity missing transformation");
                let body = RigidBodyDesc::new()
                    .mass(body_desc.mass)
                    .translation(transform.transform_vector(&Vector3::new(
                        T::zero(),
                        T::zero(),
                        T::zero(),
                    )))
                    .build();

                let body_handle = self.bodies.insert(body);

                // add all colliders associated with this body
                for (i, shape) in body_desc.shapes.iter().enumerate() {
                    let collider = ColliderDesc::new(shape.0.clone())
                        .density(shape.1)
                        .build(BodyPartHandle(body_handle, i));
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
    }
}
