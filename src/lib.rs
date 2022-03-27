use core::panic;
use egui::emath::Numeric;
use egui_wgpu_backend::RenderPass;
use gltf::{buffer, Document};
use image::DynamicImage;
use rand::{random, Rng};
use std::collections::HashMap;

use crate::{
    renderer::bindgroups::gbuffer::GBuffer,
    resources::{
        renderresult::RenderResult,
        windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent},
    },
};
//use wasm_bindgen::prelude::*;

use renderer::{
    bindgroups::{
        deferred::DeferredBindGroup, lighting::LightBindGroup, shadow::ShadowBindGroup,
        tiling::TilingBindGroup, uniforms::UniformBindGroup, HorizonBindGroup,
    },
    modelbuilder::ModelBuilder,
    pipelines::{
        forwardpipeline::ForwardPipeline, gbufferpipeline::GBufferPipeline,
        lightcullingpipeline::LightCullingPipeline, lightpipeline::LightPipeline,
        shadowpipeline::ShadowPipeline, HorizonComputePipeline, HorizonPipeline,
    },
    primitives::{lights::directionallight::DirectionalLight, uniforms::Globals},
};
use resources::{
    bindingresourcecontainer::BindingResourceContainer, camera::Camera, windowevents::ResizeEvent,
};
// #[cfg(not(target_arch = "wasm32"))]
// use scripting::scriptingengine::V8ScriptingEngine;
use specs::{Builder, Entity, EntityBuilder, Join, RunNow, World, WorldExt};

mod filesystem;
mod renderer;
mod resources;
mod scripting;
mod ui;

use crate::renderer::state::State;

mod components;
pub mod ecscontainer;
mod systems;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.

#[cfg(all(target_arch = "wasm32", feature = "wee_alloc"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use rapier3d::na::{Point3, Quaternion, UnitQuaternion, Vector3};
use ref_thread_local::RefThreadLocal;
use tokio::runtime::Runtime;
use tokio::task;
use wgpu::{BlendFactor, ColorWrites};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::components::assetidentifier::AssetIdentifier;
use crate::components::gltfmodel::{RawMaterial, RawMesh, RawModel};
use crate::filesystem::modelimporter::Importer;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::bindgroups::debugtexture::DebugTextureBindGroup;
use crate::renderer::bindgroups::material::MaterialBindGroup;
use crate::renderer::bindgroups::skybox::SkyboxBindGroup;
use crate::renderer::model::HorizonModel;
use crate::renderer::pipelines::debugtexturepipeline::DebugTexturePipeline;
use crate::renderer::pipelines::skyboxpipeline::SkyboxPipeline;
use crate::renderer::primitives::material::{GltfMaterial, MaterialUniform};
use crate::renderer::primitives::mesh::{VertexAttribValues, VertexAttributeType};
use crate::renderer::primitives::texture::Texture;
use crate::renderer::primitives::vertex::MeshVertexData;
use crate::resources::bindingresourcecontainer::BufferTypes::{
    CanvasSize, Instances, Normals, PointLight, ShadowUniform, Skybox, SpotLight, Tiling, Uniform,
};
use crate::resources::bindingresourcecontainer::SamplerTypes::{DeferredTexture, Shadow};
use crate::resources::bindingresourcecontainer::TextureViewTypes::{
    DeferredAlbedo, DeferredNormals, DeferredPosition,
};
use crate::resources::bindingresourcecontainer::{
    BufferTypes, SamplerTypes, TextureArrayViewTypes, TextureTypes, TextureViewTypes,
};
use crate::resources::camera::CameraController;
use crate::resources::defaulttexturecontainer::{DefaultTextureContainer, DefaultTextureTypes};
use crate::resources::deltatime::DeltaTime;
use crate::resources::eguicontainer::EguiContainer;
use crate::resources::projection::Projection;
use crate::resources::windowstate::WindowState;
use crate::scripting::scriptingengine::HorizonScriptingEngine;
use crate::scripting::ScriptingError;
use crate::systems::events::handlelifecycleevents::HandleInitCallbacks;
use crate::BufferTypes::{LightCulling, LightId};
use crate::TextureViewTypes::DeferredSpecular;
use ecscontainer::ECSContainer;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]

extern "C" {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
    fn call_catch(this: &wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue>;
}

// TODO: convert sender to result<T> and return proper errors
#[derive(Debug)]
pub enum CustomEvent {
    RequestModelLoad(
        HorizonModel,
        futures::channel::oneshot::Sender<Result<Entity, ScriptingError>>,
    ),
    SkyboxTextureLoad(Vec<u8>, futures::channel::oneshot::Sender<()>),
}
ref_thread_local::ref_thread_local! {
    pub static managed EVENT_LOOP_PROXY: Option<winit::event_loop::EventLoopProxy<CustomEvent>> = None;
    pub static managed ECS_CONTAINER: ECSContainer = ECSContainer::default();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
pub fn run() {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::Window;
        use winit::platform::web::WindowExtWebSys;
        let win: Window = web_sys::window().unwrap();

        let screen_x = win.inner_width().unwrap().as_f64().unwrap();
        let screen_y = win.inner_height().unwrap().as_f64().unwrap();

        log::info!(target:"window_size","x: {}", screen_x);
        log::info!(target:"window_size","y: {}", screen_y);
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("failed to initialize logger");
        let doc = win.document().unwrap();

        let body = doc.body().unwrap();
        body.style().set_property("margin", "0px").unwrap();

        let canvas = window.canvas();

        // TODO: if on web resize accordingly in the event.
        canvas.set_height(screen_y as u32);
        canvas.set_width(screen_x as u32);
        body.append_child(&web_sys::Element::from(canvas)).ok();
        doc.query_selector("canvas")
            .unwrap()
            .unwrap()
            .remove_attribute("style")
            .unwrap();

        wasm_bindgen_futures::spawn_local(async move {
            use wasm_bindgen::JsCast;
            let state = State::new(&window).await;
            let mut ecs = ECSContainer::global_mut();
            let platform = Platform::new(PlatformDescriptor {
                physical_height: state.sc_descriptor.height,
                physical_width: state.sc_descriptor.width,
                scale_factor: window.scale_factor(),
                ..Default::default()
            });
            ecs.world.insert(EguiContainer {
                render_pass: RenderPass::new(&state.device, state.sc_descriptor.format, 1),
                state: platform,
            });
            ecs.setup(state);
            setup_pipelines(&mut ecs.world);
            drop(ecs);
            // https://github.com/gfx-rs/wgpu/issues/1457
            // https://github.com/gfx-rs/wgpu/pull/1469/commits/07376d11e8b33639df3e002f2631dff27c289802

            let run_closure = Closure::once_into_js(move || {
                run(event_loop, window);
            });
            if let Err(error) = call_catch(&run_closure) {
                let is_winit_error = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                    e.message().includes("Using exceptions for control flow", 0)
                });
                if !is_winit_error {
                    web_sys::console::error_1(&error);
                }
            }
        });
    }
}
#[cfg(target_arch = "wasm32")]
pub fn run_event_loop(event_loop: EventLoop<CustomEvent>, window: winit::window::Window) {
    run_init();
    event_loop.run(move |event, _, mut control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } => {
                if window_id == window.id() {
                    handle_window_event(event, &window, control_flow);
                }
            }
            Event::DeviceEvent { ref event, .. } => {
                handle_device_events(event);
            }
            Event::RedrawRequested(_) => {
                handle_redraw_request(&window, control_flow);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::UserEvent(ref event) => {
                handle_user_events(event);
            }
            _ => {
                // run once a frame maybe?
            }
        }
    });
}

fn get_winit_resources() -> (EventLoop<CustomEvent>, Window) {
    let event_loop = EventLoop::<CustomEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    *EVENT_LOOP_PROXY.borrow_mut() = Some(proxy);

    let window = WindowBuilder::new()
        .with_title("horizon")
        .build(&event_loop)
        .unwrap();
    (event_loop, window)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn setup() -> (EventLoop<CustomEvent>, Window) {
    let winit_resources = get_winit_resources();
    env_logger::init();
    {
        let mut ecs = ECSContainer::global_mut();

        let state = State::new(&winit_resources.1).await;
        let platform = egui_winit::State::new(4096, &winit_resources.1);
        ecs.world.insert(EguiContainer {
            render_pass: RenderPass::new(&state.device, state.sc_descriptor.format, 1),
            state: platform,
            context: egui::Context::default(),
        });
        ecs.setup(state);
        setup_pipelines(&mut ecs.world);
    }
    let fut = async move {
        let ecs = ECSContainer::global();
        let mut scripting = ecs.world.write_resource::<HorizonScriptingEngine>();
        let module_id = scripting
            .js_runtime
            .load_main_module(
                &deno_core::resolve_path("./scripts/index.js").unwrap(),
                None,
            )
            .await
            .unwrap();
        let _ = scripting.js_runtime.mod_evaluate(module_id);
        scripting.js_runtime.run_event_loop(false).await.unwrap();
    };
    fut.await;
    winit_resources
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(event_loop: EventLoop<CustomEvent>, window: winit::window::Window, runtime: Runtime) {
    run_init();
    event_loop.run(move |event, _, mut control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } => {
                if window_id == window.id() {
                    handle_window_event(event, &window, control_flow);
                }
            }
            Event::DeviceEvent { ref event, .. } => {
                handle_device_events(event);
            }
            Event::RedrawRequested(_) => {
                handle_redraw_request(&window, control_flow);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::UserEvent(event) => {
                handle_user_events(event);
            }
            _ => {
                // run once a frame maybe?
                run_deno_event_loop(&runtime);
            }
        }
    });
}
fn run_init() {
    let ecs = ECSContainer::global();
    let mut run_init = HandleInitCallbacks {};
    run_init.run_now(&ecs.world); // Very nice code... really....
}
fn handle_device_events(event: &winit::event::DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        let container = ECSContainer::global();
        let mut mouse_position_event = container.world.write_resource::<MouseMoveEvent>();
        mouse_position_event.info = *delta;
        mouse_position_event.handled = false;
    }
}

fn handle_model_load(custom_event: CustomEvent) {
    if let CustomEvent::RequestModelLoad(data, sender) = custom_event {
        log::info!("got data from model load");
        let container = ECSContainer::global();
        let state = container.world.read_resource::<State>();
        let default_texture_container = container.world.read_resource::<DefaultTextureContainer>();
        let mut gpu_mats = HashMap::new();
        let mut loaded_gpu_textures: HashMap<usize, Texture> = HashMap::new();
        for (index, material_data) in &data.materials {
            material_data.upload_material_textures_to_gpu(
                &state.device,
                &state.queue,
                &data.textures,
                &mut loaded_gpu_textures,
            );
            let bind_group = material_data.register_bind_group(
                &state.device,
                &loaded_gpu_textures,
                &default_texture_container.elements,
            );
            gpu_mats.insert(
                *index,
                RawMaterial {
                    bind_group_container: bind_group,
                },
            );
        }
        let mut meshes = Vec::new();
        for mesh in &data.meshes {
            for primitive in &mesh.primitives {
                if let Some(VertexAttribValues::Float32x3(pos)) = primitive
                    .mesh
                    .vertex_attribs
                    .get(&VertexAttributeType::Position)
                {
                    let vertex_count = pos.len();
                    let mut normals = vec![[0.0, 0.0, 0.0]; vertex_count];
                    let mut tangents = vec![[1.0, 1.0, 1.0, 1.0]; vertex_count];
                    let mut vertex_colors = vec![0; vertex_count];
                    let mut texture_coords = vec![[0.0, 0.0]; vertex_count];
                    let mut weights = vec![[0.0, 0.0, 0.0, 0.0]; vertex_count];
                    let mut joint_ids = vec![0; vertex_count];

                    if let Some(VertexAttribValues::Float32x3(norm_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::Normal)
                    {
                        normals.copy_from_slice(&norm_values);
                    }
                    if let Some(VertexAttribValues::Float32x4(tangent_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::Tangent)
                    {
                        tangents.copy_from_slice(&tangent_values);
                    }

                    if let Some(VertexAttribValues::Float32x2(tex_coords_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::TextureCoords)
                    {
                        texture_coords.copy_from_slice(tex_coords_values.as_slice());
                    }

                    if let Some(VertexAttribValues::Uint32(vertex_color_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::VertexColor)
                    {
                        vertex_colors.copy_from_slice(vertex_color_values.as_slice());
                    }

                    if let Some(VertexAttribValues::Float32x4(weight_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::JointWeight)
                    {
                        weights.copy_from_slice(weight_values.as_slice())
                    }
                    if let Some(VertexAttribValues::Uint32(joint_id_values)) = primitive
                        .mesh
                        .vertex_attribs
                        .get(&VertexAttributeType::JointIndex)
                    {
                        joint_ids.copy_from_slice(joint_id_values.as_slice());
                    }
                    let mut vertex_data = Vec::new();
                    for i in 0..vertex_count {
                        vertex_data.push(MeshVertexData {
                            position: pos[i],
                            normals: normals[i],
                            tex_coords: texture_coords[i],
                            joint_index: joint_ids[i],
                            vertex_color: vertex_colors[i],
                            tangent: tangents[i],
                            joint_weight: weights[i],
                        });
                    }
                    let vertex_buffer =
                        state
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(
                                    format!("{}-vertex_buffer", primitive.mesh.name).as_str(),
                                ),
                                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                contents: bytemuck::cast_slice(vertex_data.as_slice()),
                            });
                    let indices = primitive.mesh.indices.as_ref().unwrap();
                    let index_buffer =
                        state
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(
                                    format!("{}-index_buffer", primitive.mesh.name).as_str(),
                                ),
                                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                                contents: bytemuck::cast_slice(indices.as_slice()),
                            });

                    meshes.push(RawMesh {
                        name: primitive.mesh.name.clone(),
                        index_buffer,
                        vertex_buffer,
                        material_index: primitive.material.unwrap_or(0),
                        index_buffer_len: indices.len() as u32,
                    });
                } else {
                    sender
                        .send(Err(ScriptingError::ModelLoadFailed(
                            "Error while parsing model data".to_string(),
                        )))
                        .unwrap();
                    return;
                }
            }
        }

        // let collision_builder =
        //     rapier3d::geometry::ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();
        let raw_model = RawModel {
            meshes,
            materials: gpu_mats,
        };
        let identifier = data.name.as_ref().unwrap().clone();
        let model_entity = container
            .world
            .create_entity_unchecked()
            .with(raw_model)
            .with(data)
            .with(AssetIdentifier(identifier))
            .build();

        sender.send(Ok(model_entity)).unwrap();
    }
}
fn handle_skybox_texture_override(skybox_event: CustomEvent) {
    if let CustomEvent::SkyboxTextureLoad(data, sender) = skybox_event {
        let container = ECSContainer::global();
        let state = container.world.read_resource::<State>();
        let mut binding_resource_container =
            container.world.write_resource::<BindingResourceContainer>();
        let mut bind_group_container = container.world.write_storage::<BindGroupContainer>();
        let (texture, texture_view) =
            Texture::load_skybox_texture(&state.device, &state.queue, data.as_slice());
        binding_resource_container.textures[TextureTypes::Skybox] = Some(texture);
        binding_resource_container.texture_views[TextureViewTypes::Skybox] = Some(texture_view);
        let skybox_bind_group = container.world.read_storage::<SkyboxBindGroup>();
        let (_, skybox_bind_group_container) = (&skybox_bind_group, &mut bind_group_container)
            .join()
            .next()
            .unwrap();
        *skybox_bind_group_container = SkyboxBindGroup::create_container(
            &state.device,
            (
                binding_resource_container.buffers[BufferTypes::Skybox]
                    .as_ref()
                    .unwrap(),
                binding_resource_container.texture_views[TextureViewTypes::Skybox]
                    .as_ref()
                    .unwrap(),
                binding_resource_container.samplers[SamplerTypes::Skybox]
                    .as_ref()
                    .unwrap(),
            ),
        );
        sender.send(()).unwrap();
    }
}

fn handle_redraw_request(window: &Window, control_flow: &mut winit::event_loop::ControlFlow) {
    let ecs = ECSContainer::global();
    let mut render_callbacks =
        crate::systems::events::handlelifecycleevents::HandleOnRenderCallbacks {};
    render_callbacks.run_now(&ecs.world);
    let mut egui_container = ecs.world.write_resource::<EguiContainer>();
    let inputs = egui_container.state.take_egui_input(window);
    egui_container.context.begin_frame(inputs);
    drop(egui_container);
    drop(ecs);
    let mut container = ECSContainer::global_mut();
    container.dispatch();
    drop(container);
    let container = ECSContainer::global();
    let render_result = container.world.read_resource::<RenderResult>();
    match render_result.result {
        Some(wgpu::SurfaceError::Lost) => {
            let mut resize_event = container.world.write_resource::<ResizeEvent>();
            let state = container.world.read_resource::<State>();
            resize_event.new_size = state.size;
            resize_event.handled = false;
        }
        Some(wgpu::SurfaceError::OutOfMemory) => {
            log::error!("Not enough memory to allocate new frame!");
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    };
}
fn handle_user_events(event: CustomEvent) {
    match event {
        CustomEvent::SkyboxTextureLoad(_, _) => handle_skybox_texture_override(event),
        CustomEvent::RequestModelLoad(_, _) => handle_model_load(event),
    }
}

fn handle_window_event(event: &WindowEvent, window: &Window, control_flow: &mut ControlFlow) {
    let container = ECSContainer::global();
    let mut egui = container.world.write_resource::<EguiContainer>();
    // TODO: check return value
    egui.handle_events(event);
    match event {
        WindowEvent::MouseInput { button, state, .. } => {
            let container = ECSContainer::global();
            let mut mouse_event = container.world.write_resource::<MouseInputEvent>();
            mouse_event.info = (*button, *state);
            mouse_event.handled = false;
        }
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::KeyboardInput { input, .. } => {
            if let KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Return),
                ..
            } = input
            {
                *control_flow = ControlFlow::Exit
            } else if let KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::M),
                ..
            } = input
            {
                let container = ECSContainer::global();
                let mut window_state = container.world.write_resource::<WindowState>();
                window_state.cursor_state = !window_state.cursor_state;
                window.set_cursor_grab(window_state.cursor_state).unwrap();
                window.set_cursor_visible(!window_state.cursor_state);
            }
            let container = ECSContainer::global();
            let mut keyboard_event = container.world.write_resource::<KeyboardEvent>();
            keyboard_event.info = *input;
            keyboard_event.handled = false;
        }
        WindowEvent::Resized(physical_size) => {
            let container = ECSContainer::global();
            let mut resize_event = container.world.write_resource::<ResizeEvent>();
            resize_event.new_size = *physical_size;
            resize_event.scale_factor = Some(window.scale_factor());
            resize_event.handled = false;
        }
        WindowEvent::ScaleFactorChanged {
            new_inner_size,
            scale_factor,
        } => {
            let container = ECSContainer::global();
            let mut resize_event = container.world.write_resource::<ResizeEvent>();
            resize_event.new_size = **new_inner_size;
            resize_event.scale_factor = Some(*scale_factor);
            resize_event.handled = false;
        }
        //Not working on the web currently
        WindowEvent::ModifiersChanged(_state) => {}
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_deno_event_loop(runtime: &Runtime) {
    let fut = async move {
        let ecs = ECSContainer::global();
        let mut scripting = ecs.world.write_resource::<HorizonScriptingEngine>();
        scripting.js_runtime.run_event_loop(false).await.unwrap();
    };

    let local = tokio::task::LocalSet::new();
    local.block_on(runtime, async move {
        task::spawn_local(fut).await.unwrap();
    });
}

/// Initializes a new ECS container (World) and registers the components, creates the dependency tree for the system and sets up resources.

fn setup_pipelines(world: &mut World) {
    let state = world.read_resource::<State>();
    let mut binding_resource_container = world.write_resource::<BindingResourceContainer>();
    UniformBindGroup::get_resources(&state.device, &mut binding_resource_container);
    ShadowBindGroup::get_resources(&state.device, &mut binding_resource_container);
    LightBindGroup::get_resources(&state.device, &mut binding_resource_container);
    DeferredBindGroup::get_resources(&state.device, &mut binding_resource_container);
    TilingBindGroup::get_resources(&state.device, &mut binding_resource_container);
    SkyboxBindGroup::get_resources(&state.device, &mut binding_resource_container);
    DebugTextureBindGroup::get_resources(&state.device, &mut binding_resource_container);
    GBuffer::generate_g_buffers(
        &state.device,
        &state.sc_descriptor,
        &mut binding_resource_container,
    );

    let uniform_container = UniformBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.samplers[Shadow]
                .as_ref()
                .unwrap(),
            binding_resource_container.texture_views[TextureViewTypes::Shadow]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[Uniform]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[Normals]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[Instances]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[BufferTypes::ShadowCascade]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[BufferTypes::ShadowCascadeLengths]
                .as_ref()
                .unwrap(),
        ),
    );

    let shadow_container = ShadowBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.buffers[ShadowUniform]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[Instances]
                .as_ref()
                .unwrap(),
        ),
    );

    let light_container = LightBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.buffers[BufferTypes::DirectionalLight]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[PointLight]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[SpotLight]
                .as_ref()
                .unwrap(),
        ),
    );
    let tiling_container = TilingBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.buffers[Tiling].as_ref().unwrap(),
            binding_resource_container.buffers[CanvasSize]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[LightCulling]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[LightId]
                .as_ref()
                .unwrap(),
        ),
    );

    let deferred_container = DeferredBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.samplers[DeferredTexture]
                .as_ref()
                .unwrap(),
            binding_resource_container.texture_views[DeferredPosition]
                .as_ref()
                .unwrap(),
            binding_resource_container.texture_views[DeferredNormals]
                .as_ref()
                .unwrap(),
            binding_resource_container.texture_views[DeferredAlbedo]
                .as_ref()
                .unwrap(),
            binding_resource_container.texture_views[DeferredSpecular]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[CanvasSize]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[LightId]
                .as_ref()
                .unwrap(),
            binding_resource_container.buffers[Tiling].as_ref().unwrap(),
        ),
    );
    let skybox_container = SkyboxBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.buffers[Skybox].as_ref().unwrap(),
            binding_resource_container.texture_views[TextureViewTypes::Skybox]
                .as_ref()
                .unwrap(),
            binding_resource_container.samplers[SamplerTypes::Skybox]
                .as_ref()
                .unwrap(),
        ),
    );

    let debug_texture_container = DebugTextureBindGroup::create_container(
        &state.device,
        (
            binding_resource_container.texture_views[TextureViewTypes::DeferredNormals]
                .as_ref()
                .unwrap(),
            binding_resource_container.samplers[SamplerTypes::DebugTexture]
                .as_ref()
                .unwrap(),
        ),
    );
    let gbuffer_pipeline = GBufferPipeline::create_pipeline(
        &state.device,
        (
            &crate::renderer::bindgroups::material::MaterialBindGroup::get_layout(&state.device),
            &uniform_container.layout,
        ),
        &[
            wgpu::TextureFormat::Rgba32Float.into(),
            wgpu::TextureFormat::Rgba32Float.into(),
            wgpu::TextureFormat::Rgba32Float.into(),
            wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: ColorWrites::all(),
            },
        ],
    );
    let forward_pipeline = ForwardPipeline::create_pipeline(
        &state.device,
        (
            &deferred_container.layout,
            &uniform_container.layout,
            &light_container.layout,
        ),
        &[state.sc_descriptor.format.into()],
    );

    let shadow_pipeline = ShadowPipeline::create_pipeline(
        &state.device,
        &shadow_container.layout,
        &[wgpu::TextureFormat::Depth32Float.into()],
    );

    let light_pipeline = LightPipeline::create_pipeline(
        &state.device,
        (&uniform_container.layout, &light_container.layout),
        &[state.sc_descriptor.format.into()],
    );
    let lightculling_pipeline = LightCullingPipeline::create_compute_pipeline(
        &state.device,
        (
            &uniform_container.layout,
            &light_container.layout,
            &tiling_container.layout,
        ),
    );
    let skybox_pipeline = SkyboxPipeline::create_pipeline(
        &state.device,
        &skybox_container.layout,
        &[state.sc_descriptor.format.into()],
    );

    let debug_texture_pipeline = DebugTexturePipeline::create_pipeline(
        &state.device,
        &debug_texture_container.layout,
        &[wgpu::TextureFormat::Bgra8Unorm.into()],
    );

    drop(state);
    drop(binding_resource_container);
    world.insert(LightPipeline(light_pipeline));
    world.insert(ForwardPipeline(forward_pipeline));
    world.insert(ShadowPipeline(shadow_pipeline));
    world.insert(GBufferPipeline(gbuffer_pipeline));
    world.insert(LightCullingPipeline(lightculling_pipeline));
    world.insert(SkyboxPipeline(skybox_pipeline));
    world.insert(DebugTexturePipeline(debug_texture_pipeline));
    world
        .create_entity()
        .with(UniformBindGroup)
        .with(uniform_container)
        .build();
    world
        .create_entity()
        .with(LightBindGroup)
        .with(light_container)
        .build();

    world
        .create_entity()
        .with(ShadowBindGroup)
        .with(shadow_container)
        .build();
    world
        .create_entity()
        .with(DeferredBindGroup)
        .with(deferred_container)
        .build();
    world
        .create_entity()
        .with(TilingBindGroup)
        .with(tiling_container)
        .build();
    world
        .create_entity()
        .with(SkyboxBindGroup)
        .with(skybox_container)
        .build();
    world
        .create_entity()
        .with(DebugTextureBindGroup)
        .with(debug_texture_container)
        .build();
}
