use std::{
    borrow::BorrowMut,
};

use rapier3d::{
    crossbeam::{
        self,
        channel::{Receiver},
    },
    dynamics::{
        IntegrationParameters, JointSet, RigidBody, RigidBodyHandle, RigidBodySet,
    },
    geometry::{
        BroadPhase, Collider, ColliderHandle, ColliderSet, ContactEvent, IntersectionEvent,
        NarrowPhase,
    },
    pipeline::{ChannelEventCollector, PhysicsPipeline},
};
use rapier3d::na::{UnitQuaternion, vector, Vector3};
use rapier3d::prelude::{CCDSolver, IslandManager, RigidBodyType};
use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::components::{
    physicshandle::{PhysicsHandle},
    transform::Transform,
};
use crate::ui::debugstats::DebugStats;

pub struct PhysicsWorld {
    pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    integration_parameters: IntegrationParameters,
    pub body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub joints: JointSet,
    event_handler: ChannelEventCollector,
    island_manager: IslandManager,
    gravity: Vector3<f32>,
    contact_event_receiver: Receiver<ContactEvent>,
    ccd_solver: CCDSolver,
    intersection_event_receiver: Receiver<IntersectionEvent>,
}

impl PhysicsWorld {
    pub fn new(gravity_vector: Vector3<f32>) -> Self {
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
            island_manager: IslandManager::new(),
            contact_event_receiver: contact_receive,
            intersection_event_receiver: intersection_receive,
            gravity: gravity_vector,
            ccd_solver: CCDSolver::new(),
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
            .insert_with_parent(collider_descriptor, parent_handle, &mut self.body_set)
    }
    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.body_set,
            &mut self.collider_set,
            &mut self.joints,
            &mut self.ccd_solver,
            &(),
            &self.event_handler,
        );
    }
    pub fn delete_rigid_body(&mut self,rigid_body_handle: RigidBodyHandle)
    {
        self.body_set.remove(rigid_body_handle,&mut self.island_manager,&mut self.collider_set,&mut self.joints);
    }
}

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (
        WriteExpect<'a, PhysicsWorld>,
        ReadStorage<'a, PhysicsHandle>,
        WriteStorage<'a, Transform>,
        Entities<'a>,
        WriteExpect<'a,DebugStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut world, handles, mut transforms,entities,mut debug_ui) = data;
        // perform simulation
        world.step();
        for (handle, transform,ent) in (&handles, &mut transforms,&entities).join() {
            let body = world.body_set.get(handle.rigid_body_handle).unwrap();
            transform.position = body.position().translation.vector;
            transform.rotation = body.rotation().cast();
        }
    }
}
