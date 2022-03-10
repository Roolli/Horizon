use core::panic;
use std::collections::HashMap;
use std::sync::Mutex;
use egui::emath::Numeric;
use egui_wgpu_backend::RenderPass;
use egui_winit_platform::{Platform, PlatformDescriptor};
use gltf::{buffer, Document};
use lazy_static::lazy_static;

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
        deferred::DeferredBindGroup, HorizonBindGroup, lighting::LightBindGroup,
        shadow::ShadowBindGroup, tiling::TilingBindGroup, uniforms::UniformBindGroup,
    },
    modelbuilder::ModelBuilder,
    pipelines::{
        forwardpipeline::ForwardPipeline, gbufferpipeline::GBufferPipeline,
        HorizonComputePipeline, HorizonPipeline,
        lightcullingpipeline::LightCullingPipeline, lightpipeline::LightPipeline, shadowpipeline::ShadowPipeline,
    },
    primitives::{lights::directionallight::DirectionalLight, uniforms::Globals},
};
use resources::{
    bindingresourcecontainer::BindingResourceContainer, camera::Camera, windowevents::ResizeEvent,
};
#[cfg(not(target_arch = "wasm32"))]
use scripting::scriptingengine::V8ScriptingEngine;
use specs::{Builder, Entity, EntityBuilder, Join, RunNow, World, WorldExt};

mod filesystem;
mod renderer;
mod resources;
mod scripting;
mod ui;

use crate::renderer::state::State;

mod components;
mod systems;
pub mod ecscontainer;


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

use once_cell::sync::OnceCell;
use rapier3d::na::{Point3, Vector3};
use ref_thread_local::RefThreadLocal;

use tobj::Model;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use ecscontainer::ECSContainer;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::bindgroups::skybox::SkyboxBindGroup;
use crate::renderer::model::HorizonModel;
use crate::renderer::pipelines::skyboxpipeline::SkyboxPipeline;
use crate::renderer::primitives::texture::Texture;
use crate::resources::bindingresourcecontainer::{BufferTypes, SamplerTypes, TextureTypes, TextureViewTypes};
use crate::resources::bindingresourcecontainer::BufferTypes::{CanvasSize, Instances, Normals, PointLight, ShadowUniform, Skybox, SpotLight, Tiling, Uniform};
use crate::resources::bindingresourcecontainer::SamplerTypes::{DeferredTexture, Shadow};
use crate::resources::bindingresourcecontainer::TextureViewTypes::{DeferredAlbedo, DeferredNormals, DeferredPosition};
use crate::resources::camera::CameraController;
use crate::resources::deltatime::DeltaTime;
use crate::resources::eguicontainer::EguiContainer;
use crate::resources::projection::Projection;
use crate::systems::events::handlelifecycleevents::HandleInitCallbacks;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
    fn call_catch(this: &JsValue) -> Result<(), JsValue>;
}

// TODO: convert sender to result<T> and return proper errors
#[derive(Debug)]
enum CustomEvent {
    RequestModelLoad(HorizonModel, futures::channel::oneshot::Sender<Entity>),
    SkyboxTextureLoad(Vec<u8>, futures::channel::oneshot::Sender<()>),
}
ref_thread_local::ref_thread_local! {
        pub static managed EVENT_LOOP_PROXY: Option<winit::event_loop::EventLoopProxy<CustomEvent>> = None;
        pub static managed ECS_CONTAINER: ECSContainer = ECSContainer::default();
    }

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn setup() {
    let event_loop = EventLoop::<CustomEvent>::with_user_event();
    let proxy = event_loop.create_proxy();
    *EVENT_LOOP_PROXY.borrow_mut() = Some(proxy);

    let window = WindowBuilder::new()
        .with_title("horizon")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
        {
            use web_sys::Window;
            use winit::platform::web::WindowExtWebSys;
            let win: Window = web_sys::window().unwrap();

            let screen_x = win.inner_width().unwrap().as_f64().unwrap();
            let screen_y = win.inner_height().unwrap().as_f64().unwrap();

            log::info!("x: {}", screen_x);
            log::info!("y: {}", screen_y);
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
                let _demo_app = egui_demo_lib::WrapApp::default();
                ecs.world.insert(EguiContainer {
                    render_pass: RenderPass::new(&state.device, state.sc_descriptor.format, 1),
                    platform,
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

    #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();

            unsafe {
                if !ECS_INSTANCE.set(ECSContainer::new()).is_ok() {
                    panic!();
                }

                let state = block_on(State::new(&window));
            }
            // ! for now block
            let ecs = ECSContainer::global_mut();
            ecs.setup(state);
            setup_pipelines(&mut ecs.world);
            block_on(create_debug_scene());
            run(event_loop, window);
        }
}

fn run(event_loop: EventLoop<CustomEvent>, window: winit::window::Window) {
    //TODO: Might move to state
    let mut ecs = ECSContainer::global_mut();
    ecs.world.insert(DirectionalLight::new(
        Point3::new(1.0, -1.0, 0.0),
        wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        },
    ));

    let state = ecs.world.write_resource::<State>();
    let cam = Camera::new(Point3::new(-64.0, 29.9, 0.5), f32::to_radians(-2.0), f32::to_radians(-16.0));
    let proj = Projection::new(state.sc_descriptor.width, state.sc_descriptor.height, f32::to_radians(45.0), 2.0, 200.0);
    let cam_controller = CameraController::new(10.0, 2.0);

    drop(state);
    let mut globals = Globals::new(0, 0);
    globals.update_view_proj_matrix(&cam, &proj);
    ecs.world.insert(cam_controller);
    ecs.world.insert(proj);
    ecs.world.insert(globals);
    ecs.world.insert(cam);
    let mut run_init = HandleInitCallbacks {};
    run_init.run_now(&ecs.world); // Very nice code... really....
    drop(ecs);
    event_loop.run(move |event, _, control_flow| {
        let container = ECSContainer::global();
        let mut egui_container = container.world.write_resource::<EguiContainer>();
        egui_container.platform.handle_event(&event);
        drop(egui_container);
        drop(container);
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::MouseInput { button, state, .. } => {
                        let container = ECSContainer::global();
                        let mut mouse_event = container
                            .world
                            .write_resource::<MouseInputEvent>();
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
                        }
                        let container = ECSContainer::global();
                        let mut keyboard_event = container
                            .world
                            .write_resource::<KeyboardEvent>();
                        keyboard_event.info = *input;
                        keyboard_event.handled = false;
                    }
                    WindowEvent::Resized(physical_size) => {
                        let container = ECSContainer::global();
                        let mut resize_event = container
                            .world
                            .write_resource::<ResizeEvent>();
                        resize_event.new_size = *physical_size;
                        resize_event.scale_factor = Some(window.scale_factor());
                        resize_event.handled = false;
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                        let container = ECSContainer::global();
                        let mut resize_event = container
                            .world
                            .write_resource::<ResizeEvent>();
                        resize_event.new_size = **new_inner_size;
                        resize_event.scale_factor = Some(*scale_factor);
                        resize_event.handled = false;
                    }
                    //Not working on the web currently
                    WindowEvent::ModifiersChanged(_state) => {}
                    _ => {}
                }
            }
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    let container = ECSContainer::global();
                    let mut mouse_position_event = container
                        .world
                        .write_resource::<MouseMoveEvent>();
                    mouse_position_event.info = delta;
                    mouse_position_event.handled = false;
                }
            }
            Event::RedrawRequested(_) => {
                let ecs = ECSContainer::global();
                let mut render_callbacks = crate::systems::events::handlelifecycleevents::HandleOnRenderCallbacks{};
                render_callbacks.run_now(&ecs.world);
                drop(ecs);
                let mut container = ECSContainer::global_mut();
                let mut state = container.world.write_resource::<EguiContainer>();
                let delta_time = container.world.read_resource::<DeltaTime>();
                state.platform.update_time((chrono::offset::Utc::now().timestamp_millis() - delta_time.app_start_time).to_f64() / 1000.0);
                drop(delta_time);
                drop(state);
                container.dispatch();
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
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::UserEvent(event) => {
                //TODO: add a system which handles this event and use a resource to pass the data to it!
                match event {
                    CustomEvent::SkyboxTextureLoad(data, sender) => {
                        let container = ECSContainer::global();
                        let state = container.world.read_resource::<State>();
                        let mut binding_resource_container = container.world.write_resource::<BindingResourceContainer>();
                        let mut bind_group_container = container.world.write_storage::<BindGroupContainer>();
                       let (texture,texture_view) =  Texture::load_skybox_texture(&state.device, &state.queue, data.as_slice());
                        binding_resource_container.textures[TextureTypes::Skybox] = Some(texture);
                        binding_resource_container.texture_views[TextureViewTypes::Skybox] = Some(texture_view);
                        let skybox_bind_group = container.world.read_storage::<SkyboxBindGroup>();
                        let (_,  skybox_bind_group_container) = (&skybox_bind_group, &mut bind_group_container)
                            .join()
                            .next()
                            .unwrap();
                        *skybox_bind_group_container = SkyboxBindGroup::create_container(&state.device,(binding_resource_container.buffers[BufferTypes::Skybox].as_ref().unwrap(),binding_resource_container.texture_views[TextureViewTypes::Skybox].as_ref().unwrap(),binding_resource_container.samplers[SamplerTypes::Skybox].as_ref().unwrap()));
                        sender.send(()).unwrap();
                    }
                    CustomEvent::RequestModelLoad(data, sender) => {
                        let container = ECSContainer::global();
                        let state = container.world.read_resource::<State>();
                        for material_data in data.materials
                        {
                            // TODO: continue from here!!

                        }
                        // let obj_model = container
                        //     .world
                        //     .read_resource::<ModelBuilder>().create_gltf_model();

                        // let collision_builder =
                        //     rapier3d::geometry::ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();
                        let model_entity = container
                            .world
                            .create_entity_unchecked()
                                        //.with(obj_model)
                           // .with(crate::components::modelcollider::ModelCollider(collision_builder))
                            .build();
                        sender.send(model_entity).unwrap();
                    }
                };
            }
            _ => {}
        }
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
    GBuffer::generate_g_buffers(
        &state.device,
        &state.sc_descriptor,
        &mut binding_resource_container,
    );

    let uniform_container = UniformBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .samplers[Shadow].as_ref().unwrap(),
            binding_resource_container
                .texture_views[TextureViewTypes::Shadow].as_ref().unwrap(),
            binding_resource_container
                .buffers[Uniform].as_ref().unwrap(),
            binding_resource_container
                .buffers[Normals].as_ref().unwrap(),
            binding_resource_container
                .buffers[Instances].as_ref().unwrap(),
        ),
    );

    let shadow_container = ShadowBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers[ShadowUniform].as_ref().unwrap(),
            binding_resource_container
                .buffers[Instances].as_ref().unwrap(),
        ),
    );

    let light_container = LightBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers[BufferTypes::DirectionalLight].as_ref().unwrap(),
            binding_resource_container
                .buffers[PointLight].as_ref().unwrap(),
            binding_resource_container
                .buffers[SpotLight].as_ref().unwrap(),
        ),
    );
    let tiling_container = TilingBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers[Tiling].as_ref().unwrap(),
            binding_resource_container
                .buffers[CanvasSize].as_ref().unwrap()
        ),
    );

    let deferred_container = DeferredBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .samplers[DeferredTexture].as_ref().unwrap(),
            binding_resource_container
                .texture_views[DeferredPosition].as_ref().unwrap(),
            binding_resource_container
                .texture_views[DeferredNormals].as_ref().unwrap(),
            binding_resource_container
                .texture_views[DeferredAlbedo].as_ref().unwrap(),
            binding_resource_container
                .buffers[CanvasSize].as_ref().unwrap(),
        ),
    );
    let skybox_container = SkyboxBindGroup::create_container(
        &state.device, (binding_resource_container.buffers[Skybox].as_ref().unwrap(),
                        binding_resource_container.texture_views[TextureViewTypes::Skybox].as_ref().unwrap(),
                        binding_resource_container.samplers[SamplerTypes::Skybox].as_ref().unwrap()),
    );

    let gbuffer_pipeline = GBufferPipeline::create_pipeline(
        &state.device,
        (
            &ModelBuilder::get_diffuse_texture_bind_group_layout(&state.device),
            &uniform_container.layout,
        ),
        &[
            wgpu::TextureFormat::Rgba32Float.into(),
            wgpu::TextureFormat::Rgba32Float.into(),
            wgpu::TextureFormat::Bgra8Unorm.into(),
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
    let skybox_pipeline = SkyboxPipeline::create_pipeline(&state.device, &skybox_container.layout, &[state.sc_descriptor.format.into()]);

    drop(state);
    drop(binding_resource_container);
    world.insert(LightPipeline(light_pipeline));
    world.insert(ForwardPipeline(forward_pipeline));
    world.insert(ShadowPipeline(shadow_pipeline));
    world.insert(GBufferPipeline(gbuffer_pipeline));
    world.insert(LightCullingPipeline(lightculling_pipeline));
    world.insert(SkyboxPipeline(skybox_pipeline));
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
    world.create_entity()
        .with(SkyboxBindGroup)
        .with(skybox_container)
        .build();
}

async fn create_debug_scene() {
    #[cfg(not(target_arch = "wasm32"))]
        {
            let mut js = V8ScriptingEngine::new();
            // TODO: load all the scripts and execute them before the first frame is rendered. maybe do modules and whatnot.
            js.execute(
                "test.js",
                String::from_utf8(Importer::default().import_file("./test.js").await)
                    .unwrap()
                    .as_str(),
            );

            {
                let global_context = js.global_context();
                let isolate = &mut js.isolate;

                let state_rc = V8ScriptingEngine::state(isolate);
                let js_state = state_rc.borrow();
                let handle_scope = &mut rusty_v8::HandleScope::with_context(isolate, global_context);
                for (_k, v) in js_state.callbacks.iter() {
                    let func = v.get(handle_scope);
                    let recv = rusty_v8::Integer::new(handle_scope, 1).into();
                    func.call(handle_scope, recv, &[]);
                }
            }
        }
}
