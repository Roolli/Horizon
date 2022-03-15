use __core::ops::Range;
use std::ops::{Add, Div, DivAssign};
use bytemuck::*;
use rapier3d::na::{Matrix4, Point3, Vector3, Vector4};

use wgpu::BindGroup;

use crate::{Projection, renderer::{model::HorizonModel, primitives::mesh::Mesh, state::State}, resources::camera::Camera};



pub struct DirectionalLight {
    pub direction: Point3<f32>,
    pub color: wgpu::Color,
}

impl DirectionalLight {
    pub fn new(direction: Point3<f32>, color: wgpu::Color) -> Self {
        Self { direction, color }
    }

    pub fn to_raw(&self, cam: &Camera,proj: &Projection) -> DirectionalLightRaw {
        let view_proj = Self::get_view_and_proj_matrix(self, cam, proj);
        let wgpu_view_proj =
           Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * view_proj;
        DirectionalLightRaw {
            direction: [self.direction.x, self.direction.y, self.direction.z, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
            projection: wgpu_view_proj.into(),
        }
    }
    fn get_view_and_proj_matrix(
        &self,
        cam: &Camera,
        proj:&Projection
    ) -> Matrix4<f32> {


        //TODO: get a regular projection matrix
        let view_proj_inverse = (cam.get_view_matrix()*proj.calc_proj_matrix()).try_inverse().unwrap();

        let mut corners = Vec::new();
        for x in 0..2
        {
            for y in 0..2
            {
                for z in 0..2
                {
                    let point = view_proj_inverse * Vector4::new(2.0 * (x as f32) - 1.0,2.0* y as f32 - 1.0,2.0* z as f32 - 1.0,1.0);
                    corners.push(point.component_div(&Vector4::new(1.0,1.0,1.0,1.0)));
                }
            }
        }
        let mut center = Point3::origin();
        for c in &corners {
            center += c.xyz();
        }
        let len = corners.len() as f32;

        center.div_assign(len);

        let light_view = Matrix4::look_at_rh(&Point3::new(center.x + self.direction.x,center.y + self.direction.y,center.z + self.direction.z), &self.direction,&Vector3::y());

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;
        for c in corners {
            let transform = light_view * c;
            min_x = min_x.min(transform.x);
            min_y = min_y.min(transform.y);
            min_z = min_z.min(transform.z);
            max_x = max_x.max(transform.x);
            max_y = max_y.max(transform.y);
            max_z = max_z.max(transform.z);

        }
        const ZMULT:f32 = 10.0;
        if min_z < 0.0
        {
            min_z *=ZMULT;
        }
        else {
            min_z /=ZMULT;
        }
        if max_z < 0.0
        {
            max_z /=ZMULT;
        }
        else {
            max_z *=ZMULT;
        }

        Matrix4::new_orthographic(min_x, max_x, min_y, max_y, min_z, max_z) * light_view
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct DirectionalLightRaw {
    pub projection: [[f32; 4]; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
}