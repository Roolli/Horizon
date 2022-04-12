use egui_wgpu_backend::ScreenDescriptor;
use epi::{IntegrationInfo, WebInfo};
use specs::{Join, ReadExpect, ReadStorage, System, Write, WriteExpect, WriteStorage};
use wgpu::util::DeviceExt;
use wgpu::{FilterMode, Texture, TextureView};

use crate::renderer::utils::texturerenderer::TextureRenderer;
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::eguicontainer::EguiContainer;
use crate::resources::gpuquerysets::{GpuQuerySet, GpuQuerySetContainer};
use crate::resources::surfacetexture::SurfaceTexture;
use crate::ui::debugstats::DebugStats;
use crate::ui::menu::Menu;
use crate::ui::scriptingconsole::ScriptingConsole;
use crate::ui::UiComponent;
use crate::{
    renderer::state::State, BindGroupContainer, BindingResourceContainer, BufferTypes,
    DebugTextureBindGroup, DebugTexturePipeline, DeltaTime, HorizonBindGroup, RawModel,
    SamplerTypes, TextureViewTypes,
};

pub struct RenderUIPass;

impl<'a> System<'a> for RenderUIPass {
    type SystemData = (
        WriteExpect<'a, SurfaceTexture>,
        WriteExpect<'a, EguiContainer>,
        ReadExpect<'a, State>,
        WriteExpect<'a, HorizonCommandEncoder>,
        WriteExpect<'a, DebugStats>,
        ReadExpect<'a, BindingResourceContainer>,
        ReadStorage<'a, DebugTextureBindGroup>,
        WriteStorage<'a, BindGroupContainer>,
        ReadExpect<'a, DebugTexturePipeline>,
        Write<'a, Menu>,
        Write<'a, ScriptingConsole>,
        WriteExpect<'a, GpuQuerySetContainer>,
    );

    fn run(
        &mut self,
        (
            mut surface_texture,
            mut egui_container,
            state,
            mut command_encoder,
            mut debug_ui,
            binding_resource_container,
            debug_texture_bind_group,
            mut bind_group_container,
            debug_texture_pipeline,
            mut menu_ui,
            mut console,
            mut query_sets,
        ): Self::SystemData,
    ) {
        let encoder = command_encoder.get_encoder();
        if menu_ui.show_debug_window {
            if debug_ui.debug_texture.is_none() {
                let albedo_texture: Option<Texture> =
                    Some(state.device.create_texture(&wgpu::TextureDescriptor {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        mip_level_count: 1,
                        label: Some("albedo_texture"),
                        sample_count: 1,
                        size: wgpu::Extent3d {
                            depth_or_array_layers: 1,
                            height: state.sc_descriptor.height,
                            width: state.sc_descriptor.width,
                        },
                    }));
                debug_ui.debug_texture = albedo_texture;
            }
            if debug_ui.debug_texture_view.is_none() {
                let albedo_view: Option<TextureView> =
                    Some(debug_ui.debug_texture.as_ref().unwrap().create_view(
                        &wgpu::TextureViewDescriptor {
                            array_layer_count: std::num::NonZeroU32::new(1),
                            base_array_layer: 0,
                            ..Default::default()
                        },
                    ));
                debug_ui.debug_texture_view = albedo_view;
            }
            let mut texture_view = None;
            {
                let texture = binding_resource_container.texture_views
                    [debug_ui.selected_texture_name]
                    .as_ref()
                    .unwrap();

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: debug_ui.debug_texture_view.as_ref().unwrap(),
                        resolve_target: None,
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
                    depth_stencil_attachment: None,
                    label: Some("texture renderer"),
                });

                let (_, debug_texture_bind_group_container) =
                    (&debug_texture_bind_group, &mut bind_group_container)
                        .join()
                        .next()
                        .unwrap();
                *debug_texture_bind_group_container = DebugTextureBindGroup::create_container(
                    &state.device,
                    (
                        texture,
                        binding_resource_container.samplers[SamplerTypes::DebugTexture]
                            .as_ref()
                            .unwrap(),
                    ),
                );
                render_pass.set_pipeline(&debug_texture_pipeline.0);
                render_pass.set_vertex_buffer(
                    0,
                    binding_resource_container.buffers[BufferTypes::DeferredVao]
                        .as_ref()
                        .unwrap()
                        .slice(..),
                );
                render_pass.set_bind_group(0, &debug_texture_bind_group_container.bind_group, &[]);
                render_pass.draw(0..6, 0..1);
                texture_view = Some(debug_ui.debug_texture_view.as_ref().unwrap());
            }

            if debug_ui.texture_id.is_some() {
                let id = *debug_ui.texture_id.as_ref().unwrap();
                egui_container
                    .render_pass
                    .update_egui_texture_from_wgpu_texture(
                        &state.device,
                        texture_view.as_ref().unwrap(),
                        FilterMode::Linear,
                        id,
                    )
                    .unwrap();
            } else {
                debug_ui.texture_id =
                    Some(egui_container.render_pass.egui_texture_from_wgpu_texture(
                        &state.device,
                        texture_view.as_ref().unwrap(),
                        FilterMode::Linear,
                    ));
            }

            debug_ui.show(&egui_container.context, &mut true);
        }
        if menu_ui.show_scripting_console {
            console.show(&egui_container.context, &mut true);
        }
        menu_ui.show(&egui_container.context, &mut true);
        let output = egui_container.context.end_frame();
        let paint_jobs = egui_container.context.tessellate(output.shapes);

        let screen_desc = ScreenDescriptor {
            scale_factor: state.scale_factor as f32,
            physical_height: state.sc_descriptor.height,
            physical_width: state.sc_descriptor.width,
        };

        let egui_render_pass = &mut egui_container.render_pass;

        egui_render_pass
            .add_textures(&state.device, &state.queue, &output.textures_delta)
            .unwrap();
        egui_render_pass
            .remove_textures(output.textures_delta)
            .unwrap();
        egui_render_pass.update_buffers(&state.device, &state.queue, &paint_jobs, &screen_desc);
        let output = surface_texture.texture.take().unwrap();
        let color_attachment = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut horizon_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Ui render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &color_attachment,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: None,
        });
        if let Some(ref query_set) = query_sets.container {
            horizon_render_pass
                .write_timestamp(&query_set.timestamp_queries, query_set.next_query_index * 2);
            horizon_render_pass.begin_pipeline_statistics_query(
                &query_set.pipeline_queries,
                query_set.next_query_index,
            );
        }

        egui_render_pass
            .execute_with_renderpass(&mut horizon_render_pass, &paint_jobs, &screen_desc)
            .unwrap();
        if let Some(ref query_set) = query_sets.container {
            horizon_render_pass.write_timestamp(
                &query_set.timestamp_queries,
                query_set.next_query_index * 2 + 1,
            );
            horizon_render_pass.end_pipeline_statistics_query();
        }
        drop(horizon_render_pass);

        command_encoder.finish(&state.device, &state.queue);
        output.present();
    }
}
