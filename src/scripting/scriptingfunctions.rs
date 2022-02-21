use js_sys::Error;
use crate::components::modelcollider::ModelCollider;
use crate::components::physicshandle::PhysicsHandle;
use crate::components::scriptingcallback::ScriptingCallback;
use crate::components::transform::Transform;
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::ecscontainer::ECSContainer;
use crate::systems::physics::PhysicsWorld;
use crate::{CustomEvent, EVENT_LOOP_PROXY};

use super::scriptevent::ScriptEvent;
#[cfg(not(target_arch = "wasm32"))]
use super::scriptingengine::V8ScriptingEngine;
use super::util::entityinfo::EntityInfo;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::{ColliderBuilder, ColliderHandle};
#[cfg(not(target_arch = "wasm32"))]
use rusty_v8 as v8;
use specs::prelude::*;


use rapier3d::na::{Point3, UnitQuaternion, Vector3};
use rapier3d::prelude::Isometry;

#[cfg(not(target_arch = "wasm32"))]
use v8::{Function, Global};


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::filesystem::modelimporter::Importer;
use crate::scripting::util::RigidBodyType;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]

pub struct ScriptingFunctions;

//TODO:  add custom struct to send back to js

// ! https://github.com/rustwasm/wasm-bindgen/issues/858 might need JsValue instead of function

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ScriptingFunctions {
    #[wasm_bindgen(js_name = "registerCallback")]
    pub fn register_callback(event_type: ScriptEvent, callback: js_sys::Function) {
        let mut ecs = ECSContainer::global_mut();
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
                    let rot =  component.rotation.unwrap();
                    let transform_val = Transform::new(
                        component.position.unwrap().into(),
                        UnitQuaternion::from_euler_angles(rot.x,rot.y,rot.z),
                        component.scale.unwrap().into(),
                        model_id,
                    );
                    transform = Some(transform_val);
                    builder = builder.with(transform_val);
                }
                "physics" => {
                    // Only add physics to valid already created objects
                    if let Some(model) = model_id {
                        let mut world = ecs.world.write_resource::<PhysicsWorld>();

                        let mut  rigid_body_handle:Option<RigidBodyHandle> = None;
                        let mut  collider_handle:Option<ColliderHandle> = None;
                        let pos = if let Some(val) = transform {
                            val.position
                        } else {
                            Vector3::new(0.0, 0.0, 0.0)
                        };
                        match component.body_type {
                            Some(RigidBodyType::Dynamic) =>{

                                let collider = ecs.world.read_component::<ModelCollider>();
                                let ents = ecs.world.entities();
                                for (collider_builder, entity) in (&collider, &ents).join() {
                                    if entity == model {
                                        let collider = collider_builder.0.build();
                                        let rigid_body =
                                             RigidBodyBuilder::new_dynamic()
                                            .position(Isometry::new( Vector3::new(pos.x,pos.y,pos.z), Vector3::new(0.0, 0.0, 0.0)))
                                             .additional_mass(component.mass.unwrap() as f32)
                                            .build();
                                        let body_handle = world.add_rigid_body(rigid_body);

                                        collider_handle = Some(world.add_collider(collider, body_handle));
                                        rigid_body_handle = Some(body_handle);

                                    }
                                }
                            },
                            Some(RigidBodyType::Static)=>{
                                let rigid_body = RigidBodyBuilder::new_static().position(Isometry::new(pos, Vector3::new(0.0, 0.0, 0.0)))
                                    .additional_mass(component.mass.unwrap() as f32)
                                    .build();
                                let body_handle = world.add_rigid_body(rigid_body);
                                let scale:Vector3<f32> = if let Some(scale_vals) = component.scale {
                                    scale_vals.into()
                                }
                                else {
                                   Vector3::new(1.0,1.1,1.0)
                                };
                                let collider = ColliderBuilder::cuboid(scale.x,scale.y,scale.z).build();
                                collider_handle = Some(world.add_collider(collider,body_handle));
                                rigid_body_handle = Some(body_handle);
                            },
                            _ => {
                                panic!();
                            },
                        }

                            if let (Some(body_handle), Some(collision_handle)) = (rigid_body_handle,collider_handle)
                            {
                                builder = builder.with(PhysicsHandle {
                                    collider_handle: collision_handle,
                                    rigid_body_handle:body_handle,
                                })
                            }
                    }
                }
                "pointLight" => {
                    let pos = if let Some(val) = transform {
                        val.position
                    } else {
                       Vector3::new(0.0, 0.0, 0.0)
                    };
                    let attenuation_values = component.attenuation.unwrap();
                    let color = component.color.unwrap();
                    builder = builder.with(PointLight::new(
                        Point3::from(pos),
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
    pub async fn load_model(object_name: JsValue) -> Result<JsValue,JsValue>{

        if let Some(obj) = object_name.as_string() {
            log::info!(target: "model_load","loading model {}",obj);
            let importer = Importer::default();
            let file_contents =  importer.import_obj_model(obj.as_str()).await.unwrap();

            let mut mats = Vec::new();
            for mat in file_contents.1.unwrap() {

                let diffuse_texture_raw = if !mat.diffuse_texture.is_empty()
                {
                    importer
                        .import_file(mat.diffuse_texture.as_str())
                        .await
                }
                else { Vec::new() };
                let normal_texture_raw = if !mat.normal_texture.is_empty()
                {
                    importer
                        .import_file(mat.normal_texture.as_str())
                        .await
                }
                else {
                    Vec::new()
                };
                mats.push((diffuse_texture_raw,normal_texture_raw,mat.name));
            }

                let val =ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
            let (sender,receiver) = futures::channel::oneshot::channel::<Entity>();
            val.as_ref().unwrap().send_event(CustomEvent::RequestModelLoad((file_contents.0,mats),sender)).unwrap();

           if let Ok(entity_id) = receiver.await{
               Ok(JsValue::from_f64(entity_id.id().into()))
           }
            else {
                Err(JsValue::from_str("failed to load model!"))
            }
        } else {
           Err(JsValue::from_str("Invalid model name!"))
        }
    }
    #[wasm_bindgen(js_name="setSkyboxTexture")]
    pub async fn set_skybox_texture(texture_path:JsValue) -> Result<JsValue,JsValue>
    {
        if let Some(path) = texture_path.as_string() {
           let file_contents =  Importer::default().import_file(path.as_str()).await;

            let event_loop_proxy = ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
            // MAYBE unit type is not the greatest return value...
            let (sender,receiver) = futures::channel::oneshot::channel::<()>();
            event_loop_proxy.as_ref().unwrap().send_event(CustomEvent::SkyboxTextureLoad(file_contents,sender)).unwrap();
           let res =  receiver.await;
            Ok(JsValue::TRUE)
        }
        else
        {
            Err(JsValue::from_str("Invalid argument!"))
        }
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
