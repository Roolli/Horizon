use std::ops::Deref;

use glm::Vec3;
use rapier3d::{
    dynamics::{IntegrationParameters, JointSet, RigidBody, RigidBodyHandle, RigidBodySet},
    geometry::{BroadPhase, Collider, ColliderHandle, ColliderSet, NarrowPhase},
    pipeline::PhysicsPipeline,
};
use specs::{Join, Read, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::components::{
    physicshandle::{self, PhysicsHandle},
    transform::Transform,
};

pub struct PhysicsWorld {
    pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    integration_parameters: IntegrationParameters,
    body_set: RigidBodySet,
    collider_set: ColliderSet,
    joints: JointSet,
    event_handler: (),
    gravity: Vec3,
}

impl PhysicsWorld {
    pub fn new(gravity_vector: Vec3) -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            body_set: RigidBodySet::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            collider_set: ColliderSet::new(),
            event_handler: (),
            gravity: gravity_vector,
            integration_parameters: IntegrationParameters::default(),
            joints: JointSet::new(),
        }
    }
    pub fn add_rigid_body(&mut self, rb: RigidBody) -> RigidBodyHandle {
        self.body_set.insert(rb)
    }
    pub fn add_collider(
        &mut self,
        collider_descriptor: Collider,
        parent_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        self.collider_set
            .insert(collider_descriptor, parent_handle, &mut self.body_set)
    }
    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.body_set,
            &mut self.collider_set,
            &mut self.joints,
            None,
            None,
            &self.event_handler,
        );
    }
}

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (
        WriteExpect<'a, PhysicsWorld>,
        ReadStorage<'a, PhysicsHandle>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut world, handles, mut transforms) = data;
        // perform simulation
        world.step();
        for (handle, transform) in (&handles, &mut transforms).join() {
            let body = world.body_set.get(handle.rigid_body_handle).unwrap();
            transform.position = body.position().translation.vector;
            transform.rotation = body.position().rotation.coords.into();
        }
    }
}
