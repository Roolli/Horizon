use crate::components::modelcollider::ModelCollider;
use crate::components::physicshandle::PhysicsHandle;
use crate::components::scriptingcallback::ScriptingCallback;
use crate::components::transform::Transform;
use crate::ecscontainer::{ECSContainer, ECSError};
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::systems::physics::PhysicsWorld;
use crate::{CustomEvent, EVENT_LOOP_PROXY};
use js_sys::Error;
use std::iter::Once;

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
use specs::world::Index;

#[cfg(not(target_arch = "wasm32"))]
use v8::{Function, Global};

use crate::components::assetidentifier::AssetIdentifier;
use crate::components::componentparser::{ComponentParser, ComponentParserError, ParseComponent};
use crate::components::componenttypes::ComponentTypes;
use crate::filesystem::modelimporter::Importer;
use crate::scripting::util::componentconversions::{PointLightComponent, TransformComponent};
use crate::scripting::util::horizonentity::HorizonEntity;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::scripting::util::RigidBodyType;

ref_thread_local::ref_thread_local! {
     static managed COMPONENT_PARSER: ComponentParser = ComponentParser::default();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
/// The internal struct where both runtimes call functions to execute internal mechanisms.
pub struct ScriptingFunctions;

impl ScriptingFunctions {
    // TODO: make transform mandatory!
    pub fn create_entity(entity_info: EntityInfo) -> Result<HorizonEntity, ComponentParserError> {
        let ecs = ECSContainer::global();
        let entity = ecs.world.create_entity_unchecked().build();
        let parser = ref_thread_local::RefThreadLocal::borrow(&COMPONENT_PARSER);
        parser.parse(entity_info.transform, entity, &ecs.world)?;
        for component in entity_info.components {
            parser.parse(component, entity, &ecs.world)?;
        }
        Ok(HorizonEntity::from_entity_id(entity.id()))
    }
    ///Deletes the component with the specified type for the given entity.
    pub fn delete_component(component_type: ComponentTypes, entity_id: Index) {
        let container = ECSContainer::global();
        let ent = container.world.entities().entity(entity_id);
        match component_type {
            ComponentTypes::Transform => {
                let mut transforms = container.world.write_component::<Transform>();
                transforms.remove(ent);
            }
            ComponentTypes::PhysicsHandle => {
                let mut physics_storage = container.world.write_component::<PhysicsHandle>();
                let mut physics_world = container.world.write_resource::<PhysicsWorld>();
                let physics_handle = physics_storage.get_mut(ent).unwrap();
                physics_world.delete_rigid_body(physics_handle.rigid_body_handle);
                physics_storage.remove(ent);
            }
            ComponentTypes::PointLight => {
                let mut point_light_store = container.world.write_component::<PointLight>();
                point_light_store.remove(ent);
            }
            ComponentTypes::AssetIdentifier => {
                // Not being used currently, might not be the best idea anyways to just remove identifiers,
            }
        }
    }
    pub fn insert_component(
        data: crate::scripting::util::entityinfo::Component,
        entity: Index,
    ) -> Result<(), ComponentParserError> {
        let ecs = ECSContainer::global();
        let component_parser = ref_thread_local::RefThreadLocal::borrow(&COMPONENT_PARSER);
        let world = &ecs.world;
        let entity = world.entities().entity(entity);
        component_parser.parse(data, entity, world)
    }
    /// Gets the component's data based on it's type for the given entity.
    // TODO: return some boxed stuff instead of JsValue with trait or something
    pub fn get_component(component_type: ComponentTypes, entity_id: Index) -> JsValue {
        let container = ECSContainer::global();
        match component_type {
            ComponentTypes::Transform => {
                if let Some(transform) = container
                    .world
                    .read_component::<Transform>()
                    .get(container.world.entities().entity(entity_id))
                {
                    JsValue::from_serde(&TransformComponent::from(*transform)).unwrap()
                } else {
                    JsValue::NULL
                }
            }
            ComponentTypes::PhysicsHandle => {
                if let Some(physics_handle) = container
                    .world
                    .read_component::<PhysicsHandle>()
                    .get(container.world.entities().entity(entity_id))
                {
                    JsValue::from_serde(physics_handle).unwrap()
                } else {
                    JsValue::NULL
                }
            }
            ComponentTypes::AssetIdentifier => {
                if let Some(identifier) = container
                    .world
                    .read_component::<AssetIdentifier>()
                    .get(container.world.entities().entity(entity_id))
                {
                    JsValue::from_serde(&identifier).unwrap()
                } else {
                    JsValue::NULL
                }
            }
            ComponentTypes::PointLight => {
                if let Some(point_light) = container
                    .world
                    .read_component::<PointLight>()
                    .get(container.world.entities().entity(entity_id))
                {
                    JsValue::from_serde(&PointLightComponent::from(*point_light)).unwrap()
                } else {
                    JsValue::NULL
                }
            }
        }
    }
    pub fn apply_force_to_entity(force: Vector3<f32>,entity_id: Index)
    {
        //TODO: handle non-existent physics component
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage.get(ecs.world.entities().entity(entity_id)).unwrap();
       let mut physics_world =  ecs.world.write_resource::<PhysicsWorld>();
       let body =  physics_world.body_set.get_mut(physics_handle.rigid_body_handle).unwrap();
        body.apply_force(force,true);
    }
    pub fn apply_torque_to_entity(torque: Vector3<f32>, entity_id: Index)
    {
        //TODO: handle non-existent physics component
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage.get(ecs.world.entities().entity(entity_id)).unwrap();
        let mut physics_world =  ecs.world.write_resource::<PhysicsWorld>();
        let body =  physics_world.body_set.get_mut(physics_handle.rigid_body_handle).unwrap();
        body.apply_torque(torque, true);
    }
    pub fn apply_impulse_to_entity(impulse:Vector3<f32>,entity_id: Index)
    {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage.get(ecs.world.entities().entity(entity_id)).unwrap();
        let mut physics_world =  ecs.world.write_resource::<PhysicsWorld>();
        let body =  physics_world.body_set.get_mut(physics_handle.rigid_body_handle).unwrap();
        body.apply_impulse(impulse, true);
    }
    pub fn apply_torque_impulse(torque: Vector3<f32>,entity_id: Index)
    {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage.get(ecs.world.entities().entity(entity_id)).unwrap();
        let mut physics_world =  ecs.world.write_resource::<PhysicsWorld>();
        let body =  physics_world.body_set.get_mut(physics_handle.rigid_body_handle).unwrap();
        body.apply_torque_impulse(torque, true);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "registerCallback"))]
pub fn register_callback(event_type: ScriptEvent, callback: js_sys::Function) {
    let mut ecs = ECSContainer::global_mut();
    let builder = ecs.world.create_entity();
    builder
        .with(ScriptingCallback::new(callback))
        .with(event_type)
        .build();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "loadModel"))]
pub async fn load_model(object_name: JsValue) -> Result<JsValue, JsValue> {
    if let Some(obj) = object_name.as_string() {
        log::info!(target: "model_load","loading model {}",obj);
        let importer = Importer::default();
        let file_contents = importer.import_obj_model(obj.as_str()).await.unwrap();

        let mut mats = Vec::new();
        for mat in file_contents.1.unwrap() {
            let diffuse_texture_raw = if !mat.diffuse_texture.is_empty() {
                importer.import_file(mat.diffuse_texture.as_str()).await
            } else {
                Vec::new()
            };
            let normal_texture_raw = if !mat.normal_texture.is_empty() {
                importer.import_file(mat.normal_texture.as_str()).await
            } else {
                Vec::new()
            };
            mats.push((diffuse_texture_raw, normal_texture_raw, mat.name));
        }

        let val = ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
        let (sender, receiver) = futures::channel::oneshot::channel::<Entity>();
        val.as_ref()
            .unwrap()
            .send_event(CustomEvent::RequestModelLoad(
                (file_contents.0, mats),
                sender,
            ))
            .unwrap();

        if let Ok(entity_id) = receiver.await {
            Ok(JsValue::from_f64(entity_id.id().into()))
        } else {
            Err(JsValue::from_str("failed to load model!"))
        }
    } else {
        Err(JsValue::from_str("Invalid model name!"))
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setSkyboxTexture"))]
pub async fn set_skybox_texture(texture_path: JsValue) -> Result<JsValue, JsValue> {
    if let Some(path) = texture_path.as_string() {
        let file_contents = Importer::default().import_file(path.as_str()).await;

        let event_loop_proxy = ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
        // MAYBE unit type is not the greatest return value...
        let (sender, receiver) = futures::channel::oneshot::channel::<()>();
        event_loop_proxy
            .as_ref()
            .unwrap()
            .send_event(CustomEvent::SkyboxTextureLoad(file_contents, sender))
            .unwrap();
        let res = receiver.await;
        Ok(JsValue::TRUE)
    } else {
        Err(JsValue::from_str("Invalid argument!"))
    }
}
