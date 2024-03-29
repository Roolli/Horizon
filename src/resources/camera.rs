use crate::components::transform::Transform;
use crate::renderer::state::State;
use crate::{ECSContainer, MouseInputEvent, MouseMoveEvent};
use rapier3d::na::{Isometry3, Matrix4, Point3, Vector3};
use specs::{Entity, WorldExt};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};

///FPS Style camera
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub follow_target: Option<Entity>,
    follow_target_pos: Point3<f32>,
}
impl Camera {
    /// yaw: in radians
    /// pitch: in radians
    pub fn new(pos: Point3<f32>, yaw: f32, pitch: f32) -> Self {
        Self {
            position: pos,
            pitch,
            yaw,
            follow_target: None,
            follow_target_pos: Point3::origin(),
        }
    }
    pub fn set_follow_target_ent(&mut self, target: Option<Entity>) {
        self.follow_target = target;
    }
    pub fn set_follow_target_pos(&mut self, pos: Point3<f32>) {
        self.follow_target_pos = pos;
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let f = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();
        if self.follow_target.is_some() {
            return Matrix4::look_at_rh(
                &(self.position + f),
                &self.follow_target_pos,
                &Vector3::y(),
            );
        }
        Matrix4::look_at_rh(&self.position, &(self.position + f), &Vector3::y())
    }
}
#[derive(Default)]
pub struct CameraController {
    pub move_left: f32,
    pub move_right: f32,
    pub move_forward: f32,
    pub move_backward: f32,
    pub move_up: f32,
    pub move_down: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub scroll: f32,
    pub speed: f32,
    pub sensitivity: f32,
}
impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            sensitivity,
            speed,
            ..CameraController::default()
        }
    }
    pub fn handle_keyboard_event(&mut self, input: &KeyboardInput) {
        if let Some(key_code) = input.virtual_keycode {
            let state: f32 = if input.state == ElementState::Pressed {
                1.0
            } else {
                0.0
            };
            match key_code {
                VirtualKeyCode::W => {
                    self.move_forward = state;
                }
                VirtualKeyCode::A => {
                    self.move_left = state;
                }
                VirtualKeyCode::D => {
                    self.move_right = state;
                }
                VirtualKeyCode::S => {
                    self.move_backward = state;
                }
                VirtualKeyCode::Space => {
                    self.move_up = state;
                }
                VirtualKeyCode::LShift => {
                    self.move_down = state;
                }
                _ => {}
            }
        }
    }
    pub fn handle_mouse_move(&mut self, input: &MouseMoveEvent) {
        let (delta_x, delta_y) = input.info;
        self.rotate_horizontal = delta_x as f32;
        self.rotate_vertical = delta_y as f32;
    }
    pub fn process_mouse_scroll(&mut self, input: &MouseScrollDelta) {
        self.scroll = -match input {
            MouseScrollDelta::LineDelta(data, data_2) => data_2 * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        }
    }
}
