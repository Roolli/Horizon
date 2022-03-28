use deno_core::{
    Extension, ExtensionBuilder, FsModuleLoader, JsRuntime, ModuleLoader, ModuleSource,
    ModuleSourceFuture, ModuleSpecifier, ModuleType, OpState, RuntimeOptions,
};

struct TimerPermission;

impl deno_web::TimersPermission for TimerPermission {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(&self, state: &OpState, api_name: &'static str) {}
}

use crate::components::scriptingcallback::ScriptingCallback;
use crate::scripting::scriptevent::ScriptEvent;
use crate::scripting::util::horizonentity::HorizonEntity;
use crate::{ECSContainer, Importer};
use deno_core::v8;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
use specs::{Builder, Join, WorldExt};

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct HorizonScriptingEngine;

#[cfg(not(target_arch = "wasm32"))]
pub struct HorizonScriptingEngine {
    pub js_runtime: JsRuntime,
}
#[cfg(not(target_arch = "wasm32"))]
impl Default for HorizonScriptingEngine {
    fn default() -> Self {
        let loader = std::rc::Rc::new(FsModuleLoader);
        let mut extension_builder = Extension::builder();
        let extension = HorizonScriptingEngine::add_ops(&mut extension_builder).build();
        let js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(loader),
            extensions: vec![
                deno_console::init(),
                deno_webidl::init(),
                deno_url::init(),
                deno_web::init::<TimerPermission>(BlobStore::default(), None),
                extension,
            ],

            ..Default::default()
        });
        let mut runtime = Self { js_runtime };
        runtime.create_default_module();
        runtime
    }
}

impl HorizonScriptingEngine {
    fn create_default_module(&mut self) {
        let scope = &mut self.js_runtime.handle_scope();
        let global_context = scope.get_current_context().global(scope);
        let horizon_key = v8::String::new(scope, "HorizonInternal").unwrap();
        let horizon_val = v8::Object::new(scope);
        let callback_key = v8::String::new(scope, "registerCallback").unwrap();
        let func = v8::Function::new(scope, Self::register_callback_cb).unwrap();
        func.set_name(callback_key);
        global_context.set(scope, horizon_key.into(), horizon_val.into());
        horizon_val.set(scope, callback_key.into(), func.into());
    }
    fn register_callback_cb(
        scope: &mut deno_core::v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let ecs = ECSContainer::global();

        let function = match v8::Local::<v8::Function>::try_from(args.get(0)) {
            Ok(callback) => callback,
            Err(err) => {
                log::info!("{:?}", err);
                return;
            }
        };

        let global_func: v8::Global<v8::Function> = v8::Global::new(scope, function);

        let builder = ecs.world.create_entity_unchecked();

        let event_type = ScriptEvent::from_number(args.get(1).int32_value(scope).unwrap());
        builder
            .with(ScriptingCallback::new(global_func))
            .with(event_type)
            .build();
        log::info!(target:"callbacks","callback registered!");
    }
    // perhaps a trait and a function and DI would be better where each area would be put in it's own module... yeah #notime
    fn add_ops(builder: &mut ExtensionBuilder) -> &mut ExtensionBuilder {
        builder.ops(vec![
            op_load_model::decl(),
            op_model_exists::decl(),
            op_set_skybox_texture::decl(),
            op_camera_get_pos::decl(),
            op_camera_get_yaw::decl(),
            op_camera_get_pitch::decl(),
            op_camera_set_pos::decl(),
            op_camera_set_yaw::decl(),
            op_camera_set_pitch::decl(),
            op_dir_light_get_dir::decl(),
            op_dir_light_get_color::decl(),
            op_dir_light_set_dir::decl(),
            op_dir_light_set_color::decl(),
            op_create_entity::decl(),
            op_get_component::decl(),
            op_set_component::decl(),
            op_delete_component::decl(),
            op_apply_force::decl(),
            op_apply_force_torque::decl(),
            op_apply_impulse::decl(),
            op_apply_impulse_torque::decl(),
            op_set_lin_vel::decl(),
            op_set_ang_vel::decl(),
        ])
    }
}
use crate::components::assetidentifier::AssetIdentifier;
use crate::components::componenttypes::{ComponentData, ComponentTypes};
use crate::scripting::scriptingfunctions::ScriptingFunctions;
use crate::scripting::util::entityinfo::{Component, EntityInfo};
use crate::scripting::util::glmconversion::Vec3;
use crate::scripting::util::horizonresource::{ScriptingCamera, ScriptingDirLight};
use deno_core::op;
use deno_web::BlobStore;

#[op]
async fn op_load_model(model_name: String) -> Result<u32, deno_core::anyhow::Error> {
    ScriptingFunctions::load_model(model_name)
        .await
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
        .map(|v| v.get_id())
}
#[op]
fn op_model_exists(model_name: String) -> Result<Option<HorizonEntity>, deno_core::anyhow::Error> {
    let ecs = ECSContainer::global();
    let identifiers = ecs.world.read_component::<AssetIdentifier>();
    let ents = ecs.world.entities();
    for (ent, identifier) in (&ents, &identifiers).join() {
        if identifier.0 == model_name {
            return Ok(Some(HorizonEntity::from_entity_id(ent.id())));
        }
    }
    Ok(None)
}
#[op]
async fn op_set_skybox_texture(texture_name: String) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::set_skybox_texture(texture_name)
        .await
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_camera_get_pos() -> Result<Vec3, deno_core::anyhow::Error> {
    Ok(ScriptingCamera::get_position())
}
#[op]
fn op_camera_set_pos(pos: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingCamera::set_position(pos);
    Ok(())
}
#[op]
fn op_camera_get_pitch() -> Result<f32, deno_core::anyhow::Error> {
    Ok(ScriptingCamera::get_pitch())
}
#[op]
fn op_camera_set_pitch(pitch: f32) -> Result<(), deno_core::anyhow::Error> {
    ScriptingCamera::set_pitch(pitch);
    Ok(())
}
#[op]
fn op_camera_get_yaw() -> Result<f32, deno_core::anyhow::Error> {
    Ok(ScriptingCamera::get_yaw())
}
#[op]
fn op_camera_set_yaw(yaw: f32) -> Result<(), deno_core::anyhow::Error> {
    ScriptingCamera::set_yaw(yaw);
    Ok(())
}
#[op]
fn op_camera_set_target(target:Option<u32>) -> Result<(),deno_core::anyhow::Error>
{
    ScriptingCamera::set_follow_target(target);
    Ok(())
}
#[op]
fn op_dir_light_get_dir() -> Result<Vec3, deno_core::anyhow::Error> {
    Ok(ScriptingDirLight::get_direction())
}
#[op]
fn op_dir_light_get_color() -> Result<Vec3, deno_core::anyhow::Error> {
    Ok(ScriptingDirLight::get_color())
}
#[op]
fn op_dir_light_set_dir(dir: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingDirLight::set_direction(dir);
    Ok(())
}
#[op]
fn op_dir_light_set_color(color: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingDirLight::set_color(color);
    Ok(())
}
#[op]
fn op_create_entity(entity_info: String) -> Result<u32, deno_core::anyhow::Error> {
    let entity_data = deno_core::serde_json::from_str::<EntityInfo>(entity_info.as_str())?;
    ScriptingFunctions::create_entity(entity_data)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
        .map(|v| v.get_id())
}
#[op]
fn op_get_component(
    entity_id: u32,
    component_type: u32,
) -> Result<ComponentData, deno_core::anyhow::Error> {
    Ok(ScriptingFunctions::get_component(
        component_type.into(),
        entity_id,
    ))
}
#[op]
fn op_set_component(
    entity_id: u32,
    component_data: String,
) -> Result<(), deno_core::anyhow::Error> {
    let component_data = deno_core::serde_json::from_str::<Component>(component_data.as_str())?;
    ScriptingFunctions::insert_component(component_data, entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_delete_component(
    entity_id: u32,
    component_type: ComponentTypes,
) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::delete_component(component_type, entity_id);
    Ok(())
}
#[op]
fn op_apply_force(entity_id: u32, force: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::apply_force_to_entity(force.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_apply_force_torque(
    entity_id: u32,
    force_torque: Vec3,
) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::apply_torque_to_entity(force_torque.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_apply_impulse(entity_id: u32, impulse: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::apply_impulse_to_entity(impulse.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_apply_impulse_torque(
    entity_id: u32,
    impulse_torque: Vec3,
) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::apply_torque_impulse(impulse_torque.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_set_lin_vel(entity_id: u32, lin_vel: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::set_linear_velocity(lin_vel.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
#[op]
fn op_set_ang_vel(entity_id: u32, ang_vel: Vec3) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::set_angular_velocity(ang_vel.into(), entity_id)
        .map_err(|e| deno_core::anyhow::Error::msg(format!("{:?}", e)))
}
