use __core::ops::Range;
use bytemuck::*;
use rapier3d::na::{Matrix4, Perspective3, Point3, Vector3, Vector4};
use std::ops::{Add, Div, DivAssign};

use wgpu::BindGroup;

use crate::{
    renderer::{model::HorizonModel, primitives::mesh::Mesh, state::State},
    resources::camera::Camera,
    Projection,
};

pub struct DirectionalLight {
    pub direction: Point3<f32>,
    pub color: wgpu::Color,
}

impl DirectionalLight {
    pub fn new(direction: Point3<f32>, color: wgpu::Color) -> Self {
        Self { direction, color }
    }

    pub fn to_raw(&self) -> DirectionalLightRaw {
        DirectionalLightRaw {
            direction: [self.direction.x, self.direction.y, self.direction.z, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
        }
    }
    pub fn get_view_and_proj_matrices(
        &self,
        cam: &Camera,
        z_near: f32,
        mut z_far: f32,
    ) -> Vec<(f32, Matrix4<f32>)> {
        if cfg!(target_arch = "wasm32") {
            z_far /= 10.0; // if on web reduce shadow map gen range in order to keep detail at close range.
        }
        let mut cascade_splits = Vec::new();
        let mut cascades = Vec::new();

        let clip_range = z_far - z_near;
        let min_z = z_near;
        let max_z = z_near + clip_range;
        let range = max_z - min_z;
        let ratio = max_z / min_z;
        let cascade_split_lambda = 0.95;
        // from: https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch10.html
        for i in 0..State::SHADOW_SIZE.depth_or_array_layers {
            let p = (i as f32 + 1.0) / State::SHADOW_SIZE.depth_or_array_layers as f32;
            let log = min_z * ratio.powf(p);
            let uniform = min_z + range * p;
            let d = cascade_split_lambda * (log - uniform) + uniform;
            cascade_splits.push((d - z_near) / clip_range);
        }

        let mut last_split_dist = 0.0;
        for split in cascade_splits {
            let proj = Perspective3::new(1.0, 90.0_f32.to_radians(), z_near, z_far);

            let view_proj_inverse = (proj.as_matrix() * cam.get_view_matrix())
                .try_inverse()
                .unwrap();
            let mut corners = vec![
                Vector3::new(-1.0, 1.0, -1.0),
                Vector3::new(1.0, 1.0, -1.0),
                Vector3::new(1.0, -1.0, -1.0),
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(-1.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, -1.0, 1.0),
                Vector3::new(-1.0, -1.0, 1.0),
            ];

            for corner in &mut corners {
                let inv_corner =
                    view_proj_inverse * Vector4::new(corner.x, corner.y, corner.z, 1.0);
                *corner = (inv_corner.component_div(&Vector4::new(
                    inv_corner.w,
                    inv_corner.w,
                    inv_corner.w,
                    inv_corner.w,
                )))
                .xyz();
            }

            for i in 0..4 {
                let dist = corners[i + 4] - corners[i];
                corners[i + 4] = corners[i] + (dist * split);
                corners[i] = corners[i] + (dist * last_split_dist);
            }
            // Calculate center
            let mut center = Vector3::zeros();
            for c in &corners {
                center += c.xyz();
            }
            let len = corners.len() as f32;
            center.div_assign(len);

            let mut radius = 0.0_f32;
            for corner in corners {
                let dist = (corner.xyz() - center).magnitude();
                radius = radius.max(dist);
            }
            let ceil = (radius * 16.0).ceil();
            radius = ceil / 16.0;
            let max_extent = Vector3::new(radius, radius, radius);
            let min_extent = -max_extent;
            let light_dir = -self.direction;
            let light_view = Matrix4::look_at_rh(
                &Point3::from(center - light_dir.coords * -min_extent.z),
                &Point3::from(center),
                &Vector3::y_axis(),
            );
            let ortho: Matrix4<f32> = Matrix4::new_orthographic(
                min_extent.x,
                max_extent.x,
                min_extent.y,
                max_extent.y,
                0.0_f32,
                max_extent.z - min_extent.z,
            );
            let split_depth = (z_near + split * clip_range) * -1.0;
            let view_proj = ortho * light_view;
            cascades.push((split_depth, view_proj));

            last_split_dist = split;
        }
        cascades
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct DirectionalLightRaw {
    pub direction: [f32; 4],
    pub color: [f32; 4],
}
