use crate::components::scriptingcallback::{CallbackArgs, ExecuteFunction, ScriptingCallback};
use crate::scripting::scriptevent::ScriptEvent;
use crate::scripting::util::horizonentity::HorizonEntity;
use crate::systems::physics::PhysicsWorld;
use crate::{DeltaTime, HorizonScriptingEngine};
use rapier3d::prelude::ContactEvent;
use specs::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// executes all callbacks BEFORE the event loop is started.
pub struct HandleInitCallbacks;

impl<'a> System<'a> for HandleInitCallbacks {
    type SystemData = (
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (callbacks, events, mut scripting): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::Init {
                callback.execute_with_no_args(&mut scripting);
            }
        }
    }
}

pub struct HandleOnRenderCallbacks;

impl<'a> System<'a> for HandleOnRenderCallbacks {
    type SystemData = (
        ReadExpect<'a, DeltaTime>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (dt, callbacks, events, mut scripting_engine): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::OnTick {
                callback.execute_with_args(&mut scripting_engine, CallbackArgs::Tick(dt.delta));
            }
        }
    }
}

pub struct HandleDestroyCallbacks; // Might not be needed as of now. perhaps some save/load mechanic could utilize this save all currently alive entities

pub struct InvokeEntityCollisionHandlers;

impl<'a> System<'a> for InvokeEntityCollisionHandlers {
    type SystemData = (
        ReadExpect<'a, PhysicsWorld>,
        WriteExpect<'a, HorizonScriptingEngine>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
    );

    fn run(
        &mut self,
        (physics_world, mut scripting_engine, callbacks, script_events): Self::SystemData,
    ) {
        while let Ok(recv) = physics_world.get_contact_receiver().try_recv() {
            if let ContactEvent::Started(collider1, collider2) = recv {
                for (callback, script_event) in (&callbacks, &script_events).join() {
                    if let ScriptEvent::EntityCollision = script_event {
                        let rigid_body_first = physics_world
                            .collider_set
                            .get(collider1)
                            .unwrap()
                            .parent()
                            .unwrap();
                        let rigid_body_second = physics_world
                            .collider_set
                            .get(collider2)
                            .unwrap()
                            .parent()
                            .unwrap();
                        if rigid_body_first != rigid_body_second {
                            callback.execute_with_args(
                                &mut scripting_engine,
                                CallbackArgs::EntityCollision(
                                    HorizonEntity::from_entity_id(
                                        physics_world
                                            .body_set
                                            .get(rigid_body_first)
                                            .unwrap()
                                            .user_data
                                            as u32,
                                    ),
                                    HorizonEntity::from_entity_id(
                                        physics_world
                                            .body_set
                                            .get(rigid_body_second)
                                            .unwrap()
                                            .user_data
                                            as u32,
                                    ),
                                ),
                            )
                        }
                    }
                }
            }
        }
        while let Ok(recv) = physics_world.get_intersection_receiver().try_recv() {
            if recv.intersecting {
                for (callback, script_event) in (&callbacks, &script_events).join() {
                    if let ScriptEvent::EntityCollision = script_event {
                        let first_collider =
                            physics_world.collider_set.get(recv.collider1).unwrap();
                        let first_entity_handle = if !first_collider.is_sensor() {
                            HorizonEntity::from_entity_id(
                                physics_world
                                    .body_set
                                    .get(first_collider.parent().unwrap())
                                    .unwrap()
                                    .user_data as u32,
                            )
                        } else {
                            HorizonEntity::from_entity_id(first_collider.user_data as u32)
                        };
                        let second_collider =
                            physics_world.collider_set.get(recv.collider2).unwrap();
                        let second_entity_handle = if !second_collider.is_sensor() {
                            HorizonEntity::from_entity_id(
                                physics_world
                                    .body_set
                                    .get(second_collider.parent().unwrap())
                                    .unwrap()
                                    .user_data as u32,
                            )
                        } else {
                            HorizonEntity::from_entity_id(second_collider.user_data as u32)
                        };
                        callback.execute_with_args(
                            &mut scripting_engine,
                            CallbackArgs::EntityCollision(
                                first_entity_handle,
                                second_entity_handle,
                            ),
                        )
                    }
                }
            }
        }
    }
}
