use crate::components::physicshandle::PhysicsHandle;
use crate::renderer::pipelines::debugcollision::DebugCollisionPipeline;
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::surfacetexture::SurfaceTexture;
use crate::systems::physics::PhysicsWorld;
use crate::systems::rendering::acquiretexture::AcquireTexture;
use crate::{BindGroupContainer, State, UniformBindGroup};
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

pub struct RenderCollision;

impl<'a> System<'a> for RenderCollision {
    type SystemData = (
        ReadExpect<'a, SurfaceTexture>,
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, State>,
        ReadStorage<'a, PhysicsHandle>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadExpect<'a, DebugCollisionPipeline>,
        ReadExpect<'a, PhysicsWorld>,
    );

    fn run(
        &mut self,
        (
            surface_texture,
            mut cmd_encoder,
            state,
            physics,
            uniform_bind_group_marker,
            bind_group_container,
            debug_collision_pipeline,
            physics_world,
        ): Self::SystemData,
    ) {
        let encoder = cmd_encoder.get_encoder();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                resolve_target: None,
                view: &surface_texture
                    .texture
                    .as_ref()
                    .unwrap()
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &state.depth_texture.view,
                depth_ops: None,
                stencil_ops: None,
            }),
            label: Some("collision renderer"),
        });
        let (_, uniform_bind_group_container) = (&uniform_bind_group_marker, &bind_group_container)
            .join()
            .next()
            .unwrap();
        render_pass.set_bind_group(0, &uniform_bind_group_container.bind_group, &[]);
        render_pass.set_pipeline(&debug_collision_pipeline.0);

        for (body_handle, body) in physics_world.body_set.iter() {
            for collider_handle in body.colliders() {
                let collider = physics_world.collider_set.get(*collider_handle).unwrap();
                let shape = collider.shared_shape();
                if let Some(compound_shape) = shape.as_compound() {
                    for inner_shape in compound_shape.shapes() {
                        if let Some(convex_polyhedron) = inner_shape.1.as_convex_polyhedron() {
                            let vertices = convex_polyhedron
                                .points()
                                .iter()
                                .flat_map(|v| v.coords.data.0.into_iter().flatten())
                                .collect::<Vec<_>>();
                            //TODO: add vertex buffer & create bind group with view proj and with model transform for the given collider (innershape.0)

                            // render_pass.set_vertex_buffer(0,);
                            // render_pass.draw()
                        }
                    }
                }
                if let Some(triangle_mesh) = shape.as_trimesh() {}
            }
        }
    }
}
