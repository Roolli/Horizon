use crate::components::modelcollider::ModelCollider;
use crate::components::physicshandle::PhysicsHandle;
use crate::components::scriptingcallback::ScriptingCallback;
use crate::components::transform::{self, Transform};
use crate::filesystem::modelimporter::Importer;
use crate::renderer::modelbuilder::ModelBuilder;
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::renderer::state::State;
use crate::renderer::utils::ecscontainer::ECSContainer;
use crate::systems::physics::PhysicsWorld;

use super::scriptevent::ScriptEvent;
#[cfg(not(target_arch = "wasm32"))]
use super::scriptingengine::V8ScriptingEngine;
use super::util::entityinfo::EntityInfo;
use nalgebra::Isometry3;
use rapier3d::dynamics::RigidBodyBuilder;
use rapier3d::geometry::ColliderBuilder;
#[cfg(not(target_arch = "wasm32"))]
use rusty_v8 as v8;
use specs::prelude::*;
use specs::world::Generation;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::{convert::TryFrom, io::stdout};
#[cfg(not(target_arch = "wasm32"))]
use v8::{Function, Global};
use wasm_bindgen_futures::{future_to_promise, spawn_local};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]

pub struct ScriptingFunctions;

// ! https://github.com/rustwasm/wasm-bindgen/issues/858 might need JsValue instead of function

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ScriptingFunctions {
    #[wasm_bindgen(js_name = "registerCallback")]
    pub fn register_callback(event_type: ScriptEvent, callback: js_sys::Function) {
        let ecs = ECSContainer::global_mut();
        let builder = ecs.world.create_entity();
        builder
            .with(ScriptingCallback::new(callback))
            .with(event_type)
            .build();
    }
    // Might need to change to an object instead of an array
    #[wasm_bindgen(js_name = "createEntity")]
    pub fn create_entity(entity_info: &wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
        let entity_info: EntityInfo = entity_info.into_serde().unwrap();
        let ecs = ECSContainer::global_mut();
        let mut builder: EntityBuilder = ecs.world.create_entity_unchecked();

        let mut transform: Option<Transform> = None;
        let mut model_id: Option<Entity> = None;
        for component in entity_info.components {
            match component.component_type.as_str() {
                "transform" => {
                    if let Some(model) = component.model {
                        let entities = ecs.world.entities();
                        model_id = Some(entities.entity(model));
                    }
                    let transform_val = Transform::new(
                        component.position.unwrap().into(),
                        component.rotation.unwrap().into(),
                        component.scale.unwrap().into(),
                        model_id,
                    );
                    transform = Some(transform_val);
                    builder = builder.with(transform_val);
                }
                "physics" => {
                    // Only add phyics to valid already created objects
                    if let Some(model) = model_id {
                        let mut world = ecs.world.write_resource::<PhysicsWorld>();

                        let collider = ecs.world.read_component::<ModelCollider>();
                        let ents = ecs.world.entities();
                        for (collider_builder, entity) in (&collider, &ents).join() {
                            if entity == model {
                                let pos = if let Some(val) = transform {
                                    val.position
                                } else {
                                    glm::vec3(0.0, 0.0, 0.0)
                                };

                                let rigid_body =
                                    if let Some(body_type) = component.body_type.clone() {
                                        if body_type == "DynamicRigidBody" {
                                            RigidBodyBuilder::new_dynamic()
                                        } else {
                                            RigidBodyBuilder::new_static()
                                        }
                                    } else {
                                        RigidBodyBuilder::new_static()
                                    }
                                    .position(Isometry3::new(pos, glm::vec3(0.0, 0.0, 0.0)))
                                    .mass(component.mass.unwrap() as f32)
                                    .build();
                                let rigid_body_handle = world.add_rigid_body(rigid_body);

                                let collider = collider_builder.0.build();
                                let collider_handle =
                                    world.add_collider(collider, rigid_body_handle);

                                builder = builder.with(PhysicsHandle {
                                    collider_handle,
                                    rigid_body_handle,
                                })
                            }
                        }
                    }
                }
                "pointLight" => {
                    let pos = if let Some(val) = transform {
                        val.position
                    } else {
                        glm::vec3(0.0, 0.0, 0.0)
                    };
                    let attenuation_values = component.attenuation.unwrap();
                    let color = component.color.unwrap();
                    builder = builder.with(PointLight::new(
                        pos,
                        wgpu::Color {
                            r: color.x() as f64,
                            g: color.y() as f64,
                            b: color.z() as f64,
                            a: color.w() as f64,
                        },
                        attenuation_values.x,
                        attenuation_values.y,
                        attenuation_values.z,
                    ));
                }
                _ => {}
            }
        }

        JsValue::from_f64(builder.build().id().into())
    }
    #[wasm_bindgen(js_name = "loadModel")]
    pub fn load_model(object_name: JsValue) -> js_sys::Promise {
        log::info!("erm... hello?");

        log::info!("got ecs");

        future_to_promise(async move {
            let ecs = ECSContainer::global_mut();
            let state = ecs.world.read_resource::<State>();
            let obj_model = ecs
                .world
                .read_resource::<ModelBuilder>()
                .create(
                    &state.device,
                    &state.queue,
                    object_name.as_string().unwrap().as_str(),
                )
                .await;
            log::info!("fun!!!");
            Ok(JsValue::from_f64(0.0))
        })

        //future_to_promise(async { Ok(JsValue::from_f64(0.0)) })

        //     log::info!("got ecs");
        //js_sys::Promise::resolve(&JsValue::from_f64(0.0))
        // if let Some(obj) = object_name.as_string() {
        //     let ecs = ECSContainer::global_mut();
        //     let state = ecs.world.read_resource::<State>();
        //     log::info!("got ecs");
        //     let obj_model = ecs
        //         .world
        //         .read_resource::<ModelBuilder>()
        //         .create(&state.device, &state.queue, obj.as_str())
        //         .await; // TODO: change to return result to handle missing assets
        //                 // log::info!("loaded file");
        //                 // let collision_builder =
        //                 //     ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();
        //                 // let model_entity = ecs
        //                 //     .world
        //                 //     .create_entity_unchecked()
        //                 //     .with(obj_model)
        //                 //     .with(ModelCollider(collision_builder))
        //                 //     .build();
        //                 // log::info!("success!!");
        //                 // model_entity.id()
        //     1
        // } else {
        //     //js_sys::Promise::reject(&JsValue::NULL)
        //     0
        // }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ScriptingFunctions {
    pub fn print(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let obj = args.get(0);
        let try_catch_scope = &mut v8::TryCatch::new(scope);
        let string = obj.to_string(try_catch_scope).unwrap();

        log::info!("{}", string.to_rust_string_lossy(try_catch_scope));
        stdout().flush().unwrap();
    }
    // TODO: Expose the world object and add methods for adding
    // https://github.com/denoland/deno/blob/main/core/bindings.rs#L463
    pub fn register_callback(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let mut state_rc = V8ScriptingEngine::state(scope);
        let mut state = state_rc.borrow_mut();
        let string = args.get(0).to_rust_string_lossy(scope);
        let function = match v8::Local::<v8::Function>::try_from(args.get(1)) {
            Ok(callback) => callback,
            Err(err) => {
                return;
            }
        };
        log::info!("added callback:{}", string);
        // state
        //     .callbacks
        //     .insert(string, v8::Global::new(scope, function));
    }
}
