use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::renderer::primitives::uniforms::TileInfo;
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::BufferTypes::{LightCulling, LightId};
use crate::TextureViewTypes::DeferredSpecular;
use crate::{
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{deferred::DeferredBindGroup, gbuffer::GBuffer, HorizonBindGroup},
        primitives::{texture::Texture, uniforms::CanvasConstants},
        state::State,
    },
    resources::{bindingresourcecontainer::BindingResourceContainer, windowevents::ResizeEvent},
    CanvasSize, DeferredAlbedo, DeferredNormals, DeferredPosition, DeferredTexture, Projection,
    Tiling, TilingBindGroup,
};

pub struct Resize;

impl<'a> System<'a> for Resize {
    type SystemData = (
        WriteExpect<'a, ResizeEvent>,
        WriteExpect<'a, State>,
        WriteExpect<'a, BindingResourceContainer>,
        WriteStorage<'a, BindGroupContainer>,
        WriteExpect<'a, Projection>,
        ReadStorage<'a, DeferredBindGroup>,
        ReadStorage<'a, TilingBindGroup>,
    );

    fn run(
        &mut self,
        (
            mut resize_event,
            mut state,
            mut resource_container,
            mut bind_group_container,
            mut proj,
            deferred_bind_group,
            tiling_bind_group,
        ): Self::SystemData,
    ) {
        if resize_event.handled {
            return;
        }
        // don't process 0,0 events as textures produce errors (happens on windows when minimized )
        if resize_event.new_size.height == 0 && resize_event.new_size.width == 0 {
            return;
        }

        state.size = resize_event.new_size;
        state.sc_descriptor.height = resize_event.new_size.height;
        state.sc_descriptor.width = resize_event.new_size.width;
        proj.resize(resize_event.new_size.width, resize_event.new_size.height);
        if let Some(new_scale) = resize_event.scale_factor {
            state.scale_factor = new_scale;
        }
        state.depth_texture =
            Texture::create_depth_texture(&state.device, &state.sc_descriptor, "depth_texture");
        GBuffer::generate_g_buffers(&state.device, &state.sc_descriptor, &mut resource_container);

        state.surface.configure(&state.device, &state.sc_descriptor);
        state.queue.write_buffer(
            resource_container.buffers[CanvasSize].as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&CanvasConstants {
                size: [
                    state.sc_descriptor.width as f32,
                    state.sc_descriptor.height as f32,
                ],
            }),
        );
        let mut tile_info = TileInfo::default();
        let tile_id_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            size: tile_info.calculate_light_id_buffer_size(
                state.sc_descriptor.width as f32,
                state.sc_descriptor.height as f32,
            ),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            label: Some("Light ids buffer"),
        });
        state.queue.write_buffer(
            resource_container.buffers[Tiling].as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&tile_info),
        );
        resource_container.buffers[LightId] = Some(tile_id_buffer);

        {
            let (_, deferred) = (&deferred_bind_group, &mut bind_group_container)
                .join()
                .next()
                .unwrap();
            *deferred = DeferredBindGroup::create_container(
                &state.device,
                (
                    resource_container.samplers[DeferredTexture]
                        .as_ref()
                        .unwrap(),
                    resource_container.texture_views[DeferredPosition]
                        .as_ref()
                        .unwrap(),
                    resource_container.texture_views[DeferredNormals]
                        .as_ref()
                        .unwrap(),
                    resource_container.texture_views[DeferredAlbedo]
                        .as_ref()
                        .unwrap(),
                    resource_container.texture_views[DeferredSpecular]
                        .as_ref()
                        .unwrap(),
                    resource_container.buffers[CanvasSize].as_ref().unwrap(),
                    resource_container.buffers[LightId].as_ref().unwrap(),
                    resource_container.buffers[Tiling].as_ref().unwrap(),
                ),
            );
        }

        {
            let (_, tile_bind_group) = (&tiling_bind_group, &mut bind_group_container)
                .join()
                .next()
                .unwrap();
            *tile_bind_group = TilingBindGroup::create_container(
                &state.device,
                (
                    resource_container.buffers[Tiling].as_ref().unwrap(),
                    resource_container.buffers[CanvasSize].as_ref().unwrap(),
                    resource_container.buffers[LightCulling].as_ref().unwrap(),
                    resource_container.buffers[LightId].as_ref().unwrap(),
                ),
            );
        }

        log::info!("resize has occurred!");

        resize_event.handled = true;
    }
}
