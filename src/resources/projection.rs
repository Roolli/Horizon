use rapier3d::na::Matrix4;
use crate::State;

pub struct Projection {
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Projection {
 pub fn new(width:u32,height:u32,fov_y:f32,znear:f32,zfar:f32) -> Self {
     Self {
         aspect_ratio: width as f32 / height as f32,
         z_far:zfar,
         z_near:znear,
         fov_y,
     }

 }
    pub fn resize(&mut self,width:u32,height:u32)
    {
        self.aspect_ratio = width as f32 / height as f32;
    }
    pub fn calc_proj_matrix(&self) -> Matrix4<f32>
    {
        Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * Matrix4::new_perspective(
            self.aspect_ratio,
            self.fov_y,
            self.z_near,
            self.z_far,
        )
    }
}

