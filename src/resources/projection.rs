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
        let f = 1.0 / f32::tan(self.fov_y  *0.5);
        // Infinite zfar value  https://github.com/toji/gl-matrix/commit/e906eb7bb02822a81b1d197c6b5b33563c0403c0
        let mut mat = Matrix4::zeros();
        mat[(0,0)] = f / self.aspect_ratio;
        mat[(1,1)] = f;
        mat[(2,2)] = -1.0;
        mat[(2,3)]= -self.z_near;
        mat[(3,2)] = -1.0;
        Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * mat

    }
}

