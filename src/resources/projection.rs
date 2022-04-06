use rapier3d::na::Matrix4;

pub struct Projection {
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fov_y: f32, znear: f32) -> Self {
        Self {
            aspect_ratio: width as f32 / height as f32,
            z_near: znear,
            fov_y,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }
    pub fn calc_proj_matrix(&self) -> Matrix4<f32> {
        let f = 1.0 / (self.fov_y * 0.5).tan();
        // Infinite zfar value  https://discourse.nphysics.org/t/reversed-z-and-infinite-zfar-in-projections/341
        let mut mat = Matrix4::zeros();
        mat[(0, 0)] = f / self.aspect_ratio;
        mat[(1, 1)] = f;
        mat[(2, 3)] = self.z_near;
        mat[(3, 2)] = -1.0;
        mat
    }
    pub fn calc_proj_matrix_rh_zo(&self, z_far: f32) -> Matrix4<f32> {
        let mut mat = Matrix4::zeros();
        let tan_half_fov_y = (self.fov_y * 0.5).tan();
        mat[(0, 0)] = 1.0 / (self.aspect_ratio * tan_half_fov_y);
        mat[(1, 1)] = 1.0 / tan_half_fov_y;
        mat[(2, 2)] = z_far / (self.z_near - z_far);
        mat[(2, 3)] = -(z_far * self.z_near) / (z_far - self.z_near);
        mat[(3, 2)] = -1.0;
        mat
    }
}
