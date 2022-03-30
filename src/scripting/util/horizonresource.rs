use crate::scripting::util::glmconversion::Vec3;
use crate::{DirectionalLight, ECSContainer};
#[cfg(target_arch = "wasm32")]
use js_sys::Number;
use rapier3d::na::Vector3;
use specs::WorldExt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
//TODO: add option to switch between FPS style and free cam -- done?
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "Camera"))]
pub struct ScriptingCamera;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_class = "Camera"))]
impl ScriptingCamera {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getPosition"))]
    pub fn get_position() -> Vec3 {
        Vec3::from(
            ECSContainer::global()
                .world
                .read_resource::<crate::resources::camera::Camera>()
                .position,
        )
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getYaw"))]
    /// Returns yaw in radians
    pub fn get_yaw() -> f32 {
        ECSContainer::global()
            .world
            .read_resource::<crate::resources::camera::Camera>()
            .yaw
    }
    /// Returns pitch in radians
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getPitch"))]
    pub fn get_pitch() -> f32 {
        ECSContainer::global()
            .world
            .read_resource::<crate::resources::camera::Camera>()
            .pitch
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setPosition"))]
    pub fn set_position(pos: Vec3) {
        ECSContainer::global()
            .world
            .write_resource::<crate::resources::camera::Camera>()
            .position = rapier3d::na::Point3::new(pos.x, pos.y, pos.z);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setYaw"))]
    pub fn set_yaw(yaw: f32) {
        ECSContainer::global()
            .world
            .write_resource::<crate::resources::camera::Camera>()
            .yaw = yaw;
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setPitch"))]
    pub fn set_pitch(pitch: f32) {
        ECSContainer::global()
            .world
            .write_resource::<crate::resources::camera::Camera>()
            .pitch = pitch;
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setTarget"))]
    pub fn set_follow_target(target: Option<u32>) {
        let ecs = ECSContainer::global();
        let target_ent = target.map(|t| ecs.world.entities().entity(t));
        let mut cam = ecs
            .world
            .write_resource::<crate::resources::camera::Camera>();
        cam.set_follow_target_ent(target_ent);
    }
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "DirectionalLight"))]
pub struct ScriptingDirLight;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_class = "DirectionalLight"))]
impl ScriptingDirLight {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getDirection"))]
    pub fn get_direction() -> Vec3 {
        let ecs = ECSContainer::global();
        let dir_light = ecs.world.read_resource::<DirectionalLight>();

        Vector3::new(
            dir_light.yaw.to_degrees(),
            dir_light.pitch.to_degrees(),
            0.0,
        )
        .into()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getColor"))]
    pub fn get_color() -> Vec3 {
        let ecs = ECSContainer::global();
        let color = ecs.world.read_resource::<DirectionalLight>().color;
        Vec3::new(color.r as f32, color.g as f32, color.b as f32)
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setDirection"))]
    pub fn set_direction(dir: Vec3) {
        let ecs = ECSContainer::global();
        let mut dir_light = ecs.world.write_resource::<DirectionalLight>();

        dir_light.yaw = dir.x.to_radians();
        dir_light.pitch = dir.y.to_radians();
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setColor"))]
    pub fn set_color(color: Vec3) {
        ECSContainer::global()
            .world
            .write_resource::<DirectionalLight>()
            .color = wgpu::Color {
            r: color.x as f64,
            g: color.y as f64,
            b: color.z as f64,
            a: 1.0,
        };
    }
}
