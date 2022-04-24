use crate::components::modelcollider::ModelCollider;
use crate::components::physicshandle::{PhysicsHandle, PhysicsValues};
use crate::components::scriptingcallback::ScriptingCallback;
use crate::components::transform::Transform;
use crate::ecscontainer::{ECSContainer, ECSError};
use crate::renderer::primitives::lights::pointlight::PointLight;
use crate::systems::physics::PhysicsWorld;
use crate::{CustomEvent, HorizonModel, ModelBuilder, EVENT_LOOP_PROXY};

// #[cfg(not(target_arch = "wasm32"))]
// use super::scriptingengine::V8ScriptingEngine;
use super::util::entityinfo::EntityInfo;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::{ColliderBuilder, ColliderHandle};
use specs::prelude::*;
// #[cfg(not(target_arch = "wasm32"))]
// use v8;

use rapier3d::na::{Isometry3, Point3, UnitQuaternion, Vector3};
use rapier3d::prelude::{AngVector, Isometry, RigidBody, Rotation};
use specs::world::Index;

// #[cfg(not(target_arch = "wasm32"))]
// use v8::{Function, Global};

use crate::components::assetidentifier::AssetIdentifier;
use crate::components::componentparser::{ComponentParser, ComponentParserError, ParseComponent};
use crate::components::componenttypes::{ComponentData, ComponentTypes};
use crate::scripting::util::componentconversions::{PointLightComponent, TransformComponent};
use crate::scripting::util::horizonentity::HorizonEntity;
use crate::scripting::ScriptingError;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

ref_thread_local::ref_thread_local! {
     static managed COMPONENT_PARSER: ComponentParser = ComponentParser::default();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
/// The internal struct where both runtimes call functions to execute internal mechanisms.
pub struct ScriptingFunctions;

impl ScriptingFunctions {
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
            ComponentTypes::CollisionShape => {}
            ComponentTypes::None => {}
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
    pub fn get_component(component_type: ComponentTypes, entity_id: Index) -> ComponentData {
        let container = ECSContainer::global();
        match component_type {
            ComponentTypes::Transform => {
                if let Some(transform) = container
                    .world
                    .read_component::<Transform>()
                    .get(container.world.entities().entity(entity_id))
                {
                    ComponentData::Transform(TransformComponent::from(*transform))
                } else {
                    ComponentData::Empty
                }
            }
            ComponentTypes::PhysicsHandle => {
                if let Some(physics_handle) = container
                    .world
                    .read_component::<PhysicsHandle>()
                    .get(container.world.entities().entity(entity_id))
                {
                    // all transforms (pos, rot etc.. are not returned as they are part of the transform struct and the physics system has authority over those values for entities which have physics.
                    let physics_world = container.world.read_resource::<PhysicsWorld>();
                    let rigid_body = physics_world
                        .body_set
                        .get(physics_handle.rigid_body_handle)
                        .unwrap();
                    ComponentData::Physics(PhysicsValues {
                        angular_damping: rigid_body.angular_damping(),
                        linear_damping: rigid_body.linear_damping(),
                        linear_velocity: rigid_body.linvel().xyz().into(),
                        angular_velocity: rigid_body.angvel().xyz().into(),
                        mass: rigid_body.mass(),
                    })
                } else {
                    ComponentData::Empty
                }
            }
            ComponentTypes::AssetIdentifier => {
                if let Some(identifier) = container
                    .world
                    .read_component::<AssetIdentifier>()
                    .get(container.world.entities().entity(entity_id))
                {
                    ComponentData::AssetIdentifier(identifier.0.clone())
                } else {
                    ComponentData::Empty
                }
            }
            ComponentTypes::PointLight => {
                if let Some(point_light) = container
                    .world
                    .read_component::<PointLight>()
                    .get(container.world.entities().entity(entity_id))
                {
                    ComponentData::PointLight(PointLightComponent::from(*point_light))
                } else {
                    ComponentData::Empty
                }
            }
            _ => ComponentData::Empty,
        }
    }
    pub fn apply_force_to_entity(
        force: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        let body = physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap();
        body.apply_force(force, true);
        Ok(())
    }
    pub fn apply_torque_to_entity(
        torque: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        let body = physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap();
        body.apply_torque(torque, true);
        Ok(())
    }
    pub fn apply_impulse_to_entity(
        impulse: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        let body = physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap();
        body.apply_impulse(impulse, true);
        Ok(())
    }
    pub fn apply_torque_impulse(
        torque: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap()
            .apply_torque_impulse(torque, true);
        Ok(())
    }
    pub fn set_linear_velocity(vel: Vector3<f32>, entity_id: Index) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap()
            .set_linvel(vel, true);
        Ok(())
    }
    pub fn set_angular_velocity(vel: Vector3<f32>, entity_id: Index) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let mut physics_world = ecs.world.write_resource::<PhysicsWorld>();
        physics_world
            .body_set
            .get_mut(physics_handle.rigid_body_handle)
            .unwrap()
            .set_angvel(vel, true);
        Ok(())
    }
    /// sets the entity's world position
    /// if it has physics it will set it's rigidBody's position instead (though it's still 'teleporting')
    pub fn set_entity_world_position(
        pos: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let mut transforms = ecs.world.write_storage::<Transform>();
        let entity = ecs.world.entities().entity(entity_id);
        let transform = transforms
            .get_mut(entity)
            .ok_or(ScriptingError::MissingComponent("Transform"))?;

        let physics_handles = ecs.world.read_storage::<PhysicsHandle>();
        if let Some(physics_handles) = physics_handles.get(entity) {
            let mut physics = ecs.world.write_resource::<PhysicsWorld>();
            let mut body = physics
                .body_set
                .get_mut(physics_handles.rigid_body_handle)
                .unwrap();
            body.set_position(Isometry3::new(pos, body.rotation().scaled_axis()), true);
        } else {
            transform.position = pos;
        }
        Ok(())
    }
    pub fn set_entity_rotation(
        euler_angles: Vector3<f32>,
        entity_id: Index,
    ) -> Result<(), ScriptingError> {
        let ecs = ECSContainer::global();
        let mut transforms = ecs.world.write_storage::<Transform>();
        let entity = ecs.world.entities().entity(entity_id);
        let transform = transforms
            .get_mut(entity)
            .ok_or(ScriptingError::MissingComponent("Transform"))?;

        let physics_handles = ecs.world.read_storage::<PhysicsHandle>();
        if let Some(physics_handles) = physics_handles.get(entity) {
            let mut physics = ecs.world.write_resource::<PhysicsWorld>();
            let mut body = physics
                .body_set
                .get_mut(physics_handles.rigid_body_handle)
                .unwrap();
            body.set_position(
                Isometry3::new(
                    body.position().translation.vector,
                    UnitQuaternion::from_euler_angles(
                        euler_angles[0],
                        euler_angles[1],
                        euler_angles[2],
                    )
                    .scaled_axis(),
                ),
                true,
            );
        } else {
            transform.rotation = UnitQuaternion::from_euler_angles(
                euler_angles[0],
                euler_angles[1],
                euler_angles[2],
            );
        }
        Ok(())
    }
    pub fn get_entity_forward_vector(entity_id: Index) -> Result<Vector3<f32>, ScriptingError> {
        let ecs = ECSContainer::global();
        let handle_storage = ecs.world.read_storage::<PhysicsHandle>();
        let physics_handle = handle_storage
            .get(ecs.world.entities().entity(entity_id))
            .ok_or(ScriptingError::MissingComponent("Physics"))?;
        let physics_world = ecs.world.read_resource::<PhysicsWorld>();
        let rigid_body = physics_world
            .body_set
            .get(physics_handle.rigid_body_handle)
            .unwrap();
        let direction = rigid_body.rotation().scaled_axis();
        Ok(direction)
    }

    pub async fn load_model(model_name: String) -> Result<HorizonEntity, ScriptingError> {
        log::info!(target: "model_load","loading model {}",model_name);
        let importer = crate::Importer::default();

        let gltf_contents = importer
            .import_gltf_model(model_name.as_str())
            .await
            .unwrap();
        let mut model = ModelBuilder::create_gltf_model(gltf_contents)
            .map_err(|e| {
                ScriptingError::ModelLoadFailed(format!("error during model load: {:?}", e))
            })
            .unwrap();
        model.name = Some(model_name);
        let val = ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
        let (sender, receiver) =
            futures::channel::oneshot::channel::<Result<Entity, ScriptingError>>();
        val.as_ref()
            .unwrap()
            .send_event(CustomEvent::RequestModelLoad(model, sender))
            .unwrap();
        if let Ok(res) = receiver.await {
            if let Ok(res) = res {
                Ok(HorizonEntity::from_entity_id(res.id()))
            } else {
                Err(ScriptingError::ModelLoadFailed(format!(
                    "{:?}",
                    res.err().unwrap()
                )))
            }
        } else {
            Err(ScriptingError::ModelLoadFailed(
                "failed to load model!".to_string(),
            ))
        }
    }
    pub async fn set_skybox_texture(texture_path: String) -> Result<(), ScriptingError> {
        let file_contents = crate::Importer::default()
            .import_file(texture_path.as_str())
            .await
            .map_err(|e| {
                ScriptingError::TextureOverrideFailed(format!(
                    "could not load texture:  Inner error: {}",
                    e
                ))
            })?;

        let event_loop_proxy = ref_thread_local::RefThreadLocal::borrow(&EVENT_LOOP_PROXY);
        let (sender, receiver) = futures::channel::oneshot::channel::<()>();
        event_loop_proxy
            .as_ref()
            .unwrap()
            .send_event(CustomEvent::SkyboxTextureLoad(file_contents, sender))
            .unwrap();
        let res = receiver.await;
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "registerCallback"))]
#[cfg(target_arch = "wasm32")]
pub fn register_callback(
    event_type: crate::scripting::scriptevent::ScriptEvent,
    callback: js_sys::Function,
) {
    let ecs = ECSContainer::global();
    let builder = ecs.world.create_entity_unchecked();
    builder
        .with(ScriptingCallback::new(callback))
        .with(event_type)
        .build();
}
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "loadModel"))]
pub async fn load_model(object_name: JsValue) -> Result<JsValue, JsValue> {
    if let Some(obj) = object_name.as_string() {
        ScriptingFunctions::load_model(obj)
            .await
            .map_err(|e| {
                JsValue::from_str(
                    format!("failed to override texture inner error: {:?}", e).as_str(),
                )
            })
            .map(|v| JsValue::from(v.get_id()))
    } else {
        Err(JsValue::from_str("Invalid model name!"))
    }
}
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setSkyboxTexture"))]
pub async fn set_skybox_texture(texture_path: JsValue) -> Result<JsValue, JsValue> {
    if let Some(path) = texture_path.as_string() {
        ScriptingFunctions::set_skybox_texture(path)
            .await
            .map_err(|e| {
                JsValue::from_str(
                    format!("failed to override texture inner error: {:?}", e).as_str(),
                )
            })
            .map(|v| JsValue::NULL)
    } else {
        Err(JsValue::from_str("Invalid argument!"))
    }
}
