pub struct UpdateCamera;

use crate::components::transform::Transform;
use crate::resources::deltatime::DeltaTime;
use crate::ui::debugstats::DebugStats;
use crate::{Camera, CameraController};
use rapier3d::na::{Point3, Vector3};
use specs::prelude::*;
use std::f32::consts::FRAC_PI_2;

impl UpdateCamera {
    const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
}
impl<'a> System<'a> for UpdateCamera {
    type SystemData = (
        WriteExpect<'a, Camera>,
        WriteExpect<'a, CameraController>,
        ReadExpect<'a, DeltaTime>,
        WriteExpect<'a, DebugStats>,
        ReadStorage<'a, Transform>,
    );

    fn run(
        &mut self,
        (mut camera, mut cam_controller, delta_time, mut debug_ui, transforms): Self::SystemData,
    ) {
        if let Some(target_ent) = camera.follow_target {
            if let Some(transform) = transforms.get(target_ent) {
                camera.set_follow_target_pos(Point3::from(transform.position))
            }
        }
        let dt = delta_time.delta;
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        camera.position += forward
            * (cam_controller.move_forward - cam_controller.move_backward)
            * cam_controller.speed
            * dt;
        camera.position += right
            * (cam_controller.move_right - cam_controller.move_left)
            * cam_controller.speed
            * dt;

        //zoom -  not working as no scroll event is being handled. TODO
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scroll = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position +=
            scroll * cam_controller.scroll * cam_controller.speed * cam_controller.sensitivity * dt;

        camera.position.y +=
            (cam_controller.move_up - cam_controller.move_down) * cam_controller.speed * dt;
        camera.yaw +=
            f32::to_radians(cam_controller.rotate_horizontal) * cam_controller.sensitivity * dt;
        camera.pitch +=
            f32::to_radians(-cam_controller.rotate_vertical) * cam_controller.sensitivity * dt;
        //Reset
        cam_controller.rotate_vertical = 0.0;
        cam_controller.rotate_horizontal = 0.0;
        debug_ui.cam_pos = camera.position;
        debug_ui.cam_yaw_pitch = (camera.yaw, camera.pitch);
        let clamped_value = camera
            .pitch
            .clamp(-UpdateCamera::SAFE_FRAC_PI_2, UpdateCamera::SAFE_FRAC_PI_2);
        camera.pitch = clamped_value;
    }
}
