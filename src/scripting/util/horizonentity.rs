use crate::components::componenttypes::{ComponentData, ComponentTypes};
use crate::scripting::scriptingfunctions::ScriptingFunctions;
use crate::scripting::util::entityinfo::EntityInfo;
use crate::scripting::util::glmconversion::Vec3;
use serde::Deserialize;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonEntity {
    entity_id: u32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HorizonEntity {
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(entity_info: &wasm_bindgen::JsValue) -> Result<HorizonEntity, JsValue> {
        let entity_info: EntityInfo = entity_info.into_serde().unwrap();
        ScriptingFunctions::create_entity(entity_info).map_err(|e| {
            JsValue::from_str(format!("Entity create failed with error: {:?}", e).as_str())
        })
    }
    pub fn get_id(&self) -> u32 {
        self.entity_id
    }
    pub fn from_entity_id(entity_id: u32) -> Self {
        HorizonEntity { entity_id }
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getComponent"))]
    pub fn get_component(&self, component_type: ComponentTypes) -> JsValue {
        let component_type = ScriptingFunctions::get_component(component_type, self.entity_id);
        match component_type {
            ComponentData::Transform(d) => JsValue::from_serde(&d).unwrap(),
            ComponentData::PointLight(d) => JsValue::from_serde(&d).unwrap(),
            ComponentData::AssetIdentifier(name) => JsValue::from_serde(&name).unwrap(),
            ComponentData::Physics(physics) => JsValue::from_serde(&physics).unwrap(),
            ComponentData::CollisionShape(collision) => JsValue::NULL,
            ComponentData::Empty => JsValue::NULL,
        }
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setComponent"))]
    pub fn set_component(&self, component_data: &JsValue) -> Result<(), JsValue> {
        ScriptingFunctions::insert_component(component_data.into_serde().unwrap(), self.entity_id)
            .map_err(|e| {
                JsValue::from_str(
                    format!("couldn't set component due to the following error: {:?}", e).as_str(),
                )
            })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "deleteComponent"))]
    pub fn delete_component(&self, component_type: ComponentTypes) {
        ScriptingFunctions::delete_component(component_type, self.entity_id);
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyForce"))]
    pub fn apply_force(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::apply_force_to_entity(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyForceTorque"))]
    pub fn apply_force_torque(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::apply_torque_to_entity(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyImpulse"))]
    pub fn apply_impulse(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::apply_impulse_to_entity(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyImpulseTorque"))]
    pub fn apply_impulse_torque(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::apply_torque_impulse(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setLinearVelocity"))]
    pub fn set_linear_velocity(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::set_linear_velocity(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setAngularVelocity"))]
    pub fn set_angular_velocity(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::set_angular_velocity(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setWorldPosition"))]
    pub fn set_world_position(&self, vec: Vec3) -> Result<(), JsValue> {
        ScriptingFunctions::set_entity_world_position(vec.into(), self.entity_id).map_err(|e| {
            JsValue::from_str(
                format!("failure during script execution, inner error: {:?}", e).as_str(),
            )
        })
    }
}
