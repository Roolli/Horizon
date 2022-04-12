use crate::components::physicshandle::PhysicsHandle;
use crate::components::transform::Transform;
use crate::renderer::bindgroups::debugcollision::DebugCollisionBindGroup;
use crate::renderer::pipelines::debugcollision::DebugCollisionPipeline;
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::gpuquerysets::GpuQuerySetContainer;
use crate::resources::scriptingstate::ScriptingState;
use crate::resources::surfacetexture::SurfaceTexture;
use crate::systems::physics::PhysicsWorld;
use crate::ui::debugstats::DebugStats;
use crate::ui::gpustats::Passes;
use crate::BufferTypes::{DebugCollisionUniform, DebugCollisionVertex};
use crate::{BindGroupContainer, BindingResourceContainer, State, UniformBindGroup};
use rapier3d::na::Matrix4;
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
        ReadExpect<'a, BindingResourceContainer>,
        ReadStorage<'a, DebugCollisionBindGroup>,
        ReadStorage<'a, Transform>,
        ReadExpect<'a, DebugStats>,
        WriteExpect<'a, GpuQuerySetContainer>,
    );

    fn run(
        &mut self,
        (
            surface_texture,
            mut cmd_encoder,
            state,
            physics_handles,
            uniform_bind_group_marker,
            bind_group_container,
            debug_collision_pipeline,
            physics_world,
            binding_resource_container,
            debug_collision_bind_group_marker,
            transforms,
            debug_stats,
            mut query_sets,
        ): Self::SystemData,
    ) {
        if !debug_stats.show_collision_wireframes {
            return;
        }
        let encoder = cmd_encoder.get_encoder();
        let surface_text = surface_texture.texture.as_ref().unwrap();
        let surface_view = surface_text
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                resolve_target: None,
                view: &surface_view,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        if let Some(ref query_set) = query_sets.container {
            render_pass
                .write_timestamp(&query_set.timestamp_queries, query_set.next_query_index * 2); // use manual indexing for now
            render_pass.begin_pipeline_statistics_query(
                &query_set.pipeline_queries,
                query_set.next_query_index,
            );
        }
        let (_, uniform_bind_group_container) = (&uniform_bind_group_marker, &bind_group_container)
            .join()
            .next()
            .unwrap();
        let (_, collision_uniform_bind_group) =
            (&debug_collision_bind_group_marker, &bind_group_container)
                .join()
                .next()
                .unwrap();
        render_pass.set_bind_group(0, &uniform_bind_group_container.bind_group, &[]);
        render_pass.set_pipeline(&debug_collision_pipeline.0);

        let mut vertex_count = 0;
        let alignment =
            state.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let size = std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress;
        let uniform_alignment = {
            let remainder = size % alignment;
            if remainder != 0 {
                size + alignment - remainder
            } else {
                size
            }
        };
        let mut shape_count = 0;
        for handles in physics_handles.join() {
            for collider_handle in physics_world
                .body_set
                .get(handles.rigid_body_handle)
                .unwrap()
                .colliders()
            {
                let pos = physics_world
                    .body_set
                    .get(handles.rigid_body_handle)
                    .unwrap()
                    .position();
                let collider = physics_world.collider_set.get(*collider_handle).unwrap();
                let shape = collider.shared_shape();
                if let Some(compound_shape) = shape.as_compound() {
                    for inner_shape in compound_shape.shapes().iter() {
                        if let Some(convex_polyhedron) = inner_shape.1.as_convex_polyhedron() {
                            let vertices = convex_polyhedron
                                .points()
                                .iter()
                                .flat_map(|v| (v.coords).data.0)
                                .collect::<Vec<_>>();
                            // black magic
                            let buffer = binding_resource_container.buffers[DebugCollisionVertex]
                                .as_ref()
                                .unwrap();

                            let uniform_buffer = binding_resource_container.buffers
                                [DebugCollisionUniform]
                                .as_ref()
                                .unwrap();

                            state.queue.write_buffer(
                                buffer,
                                (vertex_count * std::mem::size_of::<[f32; 3]>())
                                    as wgpu::BufferAddress,
                                bytemuck::cast_slice(&vertices),
                            );
                            let offset = (shape_count as wgpu::BufferAddress * uniform_alignment);

                            state.queue.write_buffer(
                                uniform_buffer,
                                offset,
                                bytemuck::bytes_of(&pos.to_matrix().data.0),
                            );
                            render_pass.set_bind_group(
                                1,
                                &collision_uniform_bind_group.bind_group,
                                &[offset as wgpu::DynamicOffset],
                            );
                            render_pass.set_vertex_buffer(0, buffer.slice(..));
                            render_pass.draw(
                                vertex_count as u32..(vertex_count + vertices.len()) as u32,
                                0..1,
                            );
                            render_pass.insert_debug_marker(
                                format!("draw for shape #{} ", shape_count).as_str(),
                            );
                            vertex_count += vertices.len();
                            shape_count += 1;
                        }
                    }
                }
            }
        }
        if let Some(ref mut query_set) = query_sets.container {
            render_pass.write_timestamp(
                &query_set.timestamp_queries,
                query_set.next_query_index * 2 + 1,
            );
            render_pass.end_pipeline_statistics_query();
            query_set
                .pass_indices
                .insert(Passes::Collision, query_set.next_query_index);
            query_set.next_query_index += 1;
        }
    }
}
