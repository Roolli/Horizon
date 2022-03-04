use crate::components::componenttypes::ComponentTypes;
use crate::scripting::scriptingfunctions::ScriptingFunctions;
use crate::scripting::util::entityinfo::EntityInfo;
use serde::Deserialize;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::scripting::util::glmconversion::Vec3;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonEntity {
    entity_id: u32,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HorizonEntity {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(entity_info: &wasm_bindgen::JsValue) -> Result<HorizonEntity, JsValue> {
        let entity_info: EntityInfo = entity_info.into_serde().unwrap();
        ScriptingFunctions::create_entity(entity_info).map_err(|e| {
            JsValue::from_str(format!("Entity create failed with error: {:?}", e).as_str())
        })
    }

    pub fn from_entity_id(entity_id: u32) -> Self {
        HorizonEntity { entity_id }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getComponent"))]
    pub fn get_component(&self, component_type: ComponentTypes) -> JsValue {
        ScriptingFunctions::get_component(component_type, self.entity_id)
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setComponent"))]
    pub fn set_component(&self, component_data: &JsValue) {
        ScriptingFunctions::insert_component(component_data.into_serde().unwrap(), self.entity_id)
            .unwrap(); //TODO: return err
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "deleteComponent"))]
    pub fn delete_component(&self, component_type: ComponentTypes) {
        ScriptingFunctions::delete_component(component_type, self.entity_id);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyForce"))]
    pub fn apply_force(&self,vec: &JsValue){
        //TODO: error check
        ScriptingFunctions::apply_force_to_entity(vec.into_serde::<Vec3>().unwrap().into(),self.entity_id);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyForceTorque"))]
    pub fn apply_force_torque(&self,vec:&JsValue)
    {
        ScriptingFunctions::apply_torque_to_entity(vec.into_serde::<Vec3>().unwrap().into(),self.entity_id);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyImpulse"))]
    pub fn apply_impulse(&self,vec:&JsValue)
    {
        ScriptingFunctions::apply_impulse_to_entity(vec.into_serde::<Vec3>().unwrap().into(),self.entity_id);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "applyImpulseTorque"))]
    pub fn apply_impulse_torque(&self,vec: &JsValue)
    {
        ScriptingFunctions::apply_torque_impulse(vec.into_serde::<Vec3>().unwrap().into(),self.entity_id);
    }

}
