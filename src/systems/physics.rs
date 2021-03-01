use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    ops::Deref,
    time::Instant,
};

use glm::Vec3;
use rapier3d::{
    crossbeam::{
        self,
        channel::{Receiver, Sender},
    },
    dynamics::{
        BodyStatus, IntegrationParameters, JointSet, RigidBody, RigidBodyHandle, RigidBodySet,
    },
    geometry::{
        BroadPhase, Collider, ColliderHandle, ColliderSet, ContactEvent, IntersectionEvent,
        NarrowPhase,
    },
    pipeline::{ChannelEventCollector, PhysicsPipeline},
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
    pub body_set: RigidBodySet,
    collider_set: ColliderSet,
    joints: JointSet,
    event_handler: ChannelEventCollector,
    gravity: Vec3,
    contact_event_receiver: Receiver<ContactEvent>,
    intersection_event_receiver: Receiver<IntersectionEvent>,
}

impl PhysicsWorld {
    pub fn new(gravity_vector: Vec3) -> Self {
        let (contact_send, contact_receive) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_receive) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(intersection_send, contact_send);
        Self {
            pipeline: PhysicsPipeline::new(),
            body_set: RigidBodySet::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            collider_set: ColliderSet::new(),
            event_handler,
            contact_event_receiver: contact_receive,
            intersection_event_receiver: intersection_receive,
            gravity: gravity_vector,
            integration_parameters: IntegrationParameters {
                ..Default::default()
            },
            joints: JointSet::new(),
        }
    }

    pub fn get_contact_receiver(&self) -> &Receiver<ContactEvent> {
        &self.contact_event_receiver
    }
    pub fn get_intersection_receiver(&self) -> &Receiver<IntersectionEvent> {
        &self.intersection_event_receiver
    }
    pub fn add_rigid_body(&mut self, rb: RigidBody) -> RigidBodyHandle {
        self.body_set.borrow_mut().insert(rb)
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
