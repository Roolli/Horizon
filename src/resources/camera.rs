use rapier3d::na::{Isometry3, Matrix4, Point3, Vector3};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
use crate::{MouseInputEvent, MouseMoveEvent};
use crate::renderer::state::State;
///FPS Style camera
pub struct Camera {

    pub position: Point3<f32>,
    pub yaw:f32,
    pub pitch:f32,

}
impl Camera {

    /// yaw: in radians
    /// pitch: in radians
    pub fn new(pos:Point3<f32>,yaw:f32,pitch:f32)-> Self {
        Self {
            position:pos,
            pitch,
            yaw
        }
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32>
    {
        let f = Vector3::new(self.yaw.cos(),self.pitch.sin(),self.yaw.sin()).normalize();
        Matrix4::look_at_rh(&self.position,&(self.position +f),&Vector3::y())
        //Isometry3::face_towards(&self.position,&(self.position + Vector3::new(0.0,0.0,-1.0)),&Vector3::y()).to_homogeneous()
       // let mat = Matrix4::new(
       //      s.x,u.x,-f.x,0.0,
       //      s.y, u.y,-f.y,0.0,
       //      s.z,u.z,-f.z,0.0,
       //      -eye.dot(&s),-eye.dot(&u),eye.dot(&f),1.0
       //
       //  );
        //Matrix4::new_translation(&eye) * Matrix4::new_rotation(0,&self.yaw,)
       // mat
    }
}
#[derive(Default)]
pub struct CameraController
{
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
impl CameraController
{
    pub fn new(speed:f32,sensitivity:f32)-> Self {
        Self {
            sensitivity,
            speed,
            ..CameraController::default()
        }
    }
    pub fn handle_keyboard_event(&mut self,input: &KeyboardInput)
    {
        if let Some(key_code) = input.virtual_keycode
        {

            let state:f32  = if input.state == ElementState::Pressed {
                1.0
            }
            else
            {
                0.0
            };
            log::info!("keyCode {:?} state: {:?}",key_code,input.state);
            match key_code {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    self.move_forward = state;
                },
                VirtualKeyCode::A |VirtualKeyCode::Left =>{
                    self.move_left = state;
                },
                VirtualKeyCode::D | VirtualKeyCode::Right =>{
                    self.move_right = state;
                },
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    self.move_backward = state;
                },
                VirtualKeyCode::Space  =>{
                    self.move_up = state;
                },
                VirtualKeyCode::LShift => {
                    self.move_down = state;
                },
                _ =>{}
            }
        }

    }
    pub fn handle_mouse_move(&mut self,input: &MouseMoveEvent)
    {
        let (delta_x,delta_y) = input.info;
        self.rotate_horizontal = delta_x as f32;
        self.rotate_vertical = delta_y as f32;
    }
    pub fn process_mouse_scroll(&mut self,input:&MouseScrollDelta)
    {
        self.scroll = -match input {
            MouseScrollDelta::LineDelta(data,data_2) => {data_2 * 100.0},
            MouseScrollDelta::PixelDelta(PhysicalPosition{y:scroll,..}) => {
                *scroll as f32
            }
        }
    }
}