use core::panic;

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
    utils::ecscontainer::ECSContainer,
};
use resources::{
    bindingresourcecontainer::BindingResourceContainer, camera::Camera, windowevents::ResizeEvent,
};
#[cfg(not(target_arch = "wasm32"))]
use scripting::scriptingengine::V8ScriptingEngine;
use specs::{Builder, Join, World, WorldExt};

mod filesystem;
mod renderer;
mod resources;
mod scripting;

use crate::renderer::state::State;
mod components;
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

use once_cell::sync::OnceCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

static mut ECS_INSTANCE: OnceCell<ECSContainer> = OnceCell::new();
static EVENT_LOOP_STARTED: OnceCell<bool> = OnceCell::new();

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch,js_namespace=Function,js_name="prototype.call.call")]
    fn call_catch(this: &JsValue) -> Result<(), JsValue>;
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn setup() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("horizon")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("failed to initialize logger");
        use web_sys::Window;
        use winit::platform::web::WindowExtWebSys;
        let win: Window = web_sys::window().unwrap();

        let screen_x = win.inner_width().unwrap().as_f64().unwrap();

        let screen_y = win.inner_height().unwrap().as_f64().unwrap();
        log::info!("x: {}", screen_x);
        log::info!("y: {}", screen_y);
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
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm_bindgen_futures::spawn_local(async move {
            use wasm_bindgen::JsCast;
            let state = State::new(&window).await;
            let ecs = ECSContainer::global_mut();
            ecs.setup(state);
            setup_pipelines(&mut ecs.world);
            create_debug_scene().await;
            // https://github.com/gfx-rs/wgpu/issues/1457
            // https://github.com/gfx-rs/wgpu/pull/1469/commits/07376d11e8b33639df3e002f2631dff27c289802

            let run_closure = Closure::once_into_js(move || {
                run(event_loop, window);
            });
            if let Err(error) = call_catch(&run_closure) {
                let is_winit_error = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                    e.message().includes("using exceptions for control flow", 0)
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "initECS"))]
pub fn init_ecs() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        if ECS_INSTANCE.set(ECSContainer::new()).is_err() {
            panic!();
        }
    }
}

fn run(event_loop: EventLoop<()>, window: winit::window::Window) {
    log::info!("running event loop");
    EVENT_LOOP_STARTED.set(true).unwrap();
    event_loop.run(move |event, _, control_flow| {
        // take ownership of ecs
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::MouseInput { button, state, .. } => {
                        let mut mouse_event = ECSContainer::global_mut()
                            .world
                            .write_resource::<MouseInputEvent>();
                        mouse_event.info = (*button, *state);
                        mouse_event.handled = false;
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input
                        {
                            *control_flow = ControlFlow::Exit
                        }
                        let mut keyboard_event = ECSContainer::global_mut()
                            .world
                            .write_resource::<KeyboardEvent>();
                        keyboard_event.info = *input;
                        keyboard_event.handled = false;
                    }
                    WindowEvent::Resized(physical_size) => {
                        let mut resize_event = ECSContainer::global_mut()
                            .world
                            .write_resource::<ResizeEvent>();
                        resize_event.new_size = *physical_size;
                        resize_event.handled = false;
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let mut resize_event = ECSContainer::global_mut()
                            .world
                            .write_resource::<ResizeEvent>();
                        resize_event.new_size = **new_inner_size;
                        resize_event.handled = false;
                    }
                    //Not working on the web currently
                    WindowEvent::ModifiersChanged(_state) => {}
                    _ => {}
                }
            }
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    let mut mouse_position_event = ECSContainer::global_mut()
                        .world
                        .write_resource::<MouseMoveEvent>();
                    mouse_position_event.info = delta;
                    mouse_position_event.handled = false;
                }
            }
            Event::RedrawRequested(_) => {
                let container = ECSContainer::global_mut();
                container.dispatcher.dispatch(&container.world);
                //TODO: handle events related to wgpu errors
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
            _ => {}
        }
    });
}
/// Initializes a new ECS container (World) and registers the components, creates the dependency chain for the system and sets up resources.

fn setup_pipelines(world: &mut World) {
    let state = world.read_resource::<State>();
    let mut binding_resource_container = world.write_resource::<BindingResourceContainer>();
    UniformBindGroup::get_resources(&state.device, &mut binding_resource_container);
    ShadowBindGroup::get_resources(&state.device, &mut binding_resource_container);
    LightBindGroup::get_resources(&state.device, &mut binding_resource_container);
    DeferredBindGroup::get_resources(&state.device, &mut binding_resource_container);
    TilingBindGroup::get_resources(&state.device, &mut binding_resource_container);
    GBuffer::generate_g_buffers(
        &state.device,
        &state.sc_descriptor,
        &mut binding_resource_container,
    );

    let uniform_container = UniformBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .samplers
                .get("shadow_sampler")
                .unwrap(),
            binding_resource_container
                .texture_views
                .get("shadow_view")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("uniform_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("normal_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("instance_buffer")
                .unwrap(),
        ),
    );

    let shadow_container = ShadowBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers
                .get("shadow_uniform_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("instance_buffer")
                .unwrap(),
        ),
    );

    let light_container = LightBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers
                .get("directional_light_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("point_light_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("spot_light_buffer")
                .unwrap(),
        ),
    );
    let tiling_container = TilingBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .buffers
                .get("tiling_buffer")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("canvas_size_buffer")
                .unwrap(),
        ),
    );

    let deferred_container = DeferredBindGroup::create_container(
        &state.device,
        (
            binding_resource_container
                .samplers
                .get("texture_sampler")
                .unwrap(),
            binding_resource_container
                .texture_views
                .get("position_view")
                .unwrap(),
            binding_resource_container
                .texture_views
                .get("normal_view")
                .unwrap(),
            binding_resource_container
                .texture_views
                .get("albedo_view")
                .unwrap(),
            binding_resource_container
                .buffers
                .get("canvas_size_buffer")
                .unwrap(),
        ),
    );

    let gbuffer_pipeline = GBufferPipeline::create_pipeline(
        &state.device,
        (
            &world
                .read_resource::<ModelBuilder>()
                .diffuse_texture_bind_group_layout,
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

    drop(state);
    drop(binding_resource_container);
    world.insert(LightPipeline(light_pipeline));
    world.insert(ForwardPipeline(forward_pipeline));
    world.insert(ShadowPipeline(shadow_pipeline));
    world.insert(GBufferPipeline(gbuffer_pipeline));
    world.insert(LightCullingPipeline(lightculling_pipeline));
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

    let ecs = ECSContainer::global_mut();
    const NUM_INSTANCES_PER_ROW: u32 = 0;
    ecs.world.insert(DirectionalLight::new(
        glm::vec3(1.0, -1.0, 0.0),
        wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        },
    ));

    let state = ecs.world.write_resource::<State>();
    // CAMERA
    let cam = Camera {
        eye: glm::vec3(-10.0, 15.0, 10.0),
        target: glm::vec3(0.0, 0.0, 0.0),
        up: glm::vec3(0.0, 1.0, 0.0), // Unit Y vector
        aspect_ratio: state.sc_descriptor.width as f32 / state.sc_descriptor.height as f32,
        fov_y: 90.0,
        z_near: 0.1,
        z_far: 200.0,
    };

    //    MODEL LOADING

    // let obj_model = ecs
    //     .world
    //     .read_resource::<ModelBuilder>()
    //     .create(&state.device, &state.queue, "cube.obj")
    //     .await;

    drop(state);
    let mut globals = Globals::new(0, 0);
    globals.update_view_proj_matrix(&cam);

    ecs.world.insert(globals);
    ecs.world.insert(cam);

    // let collision_builder =
    //     ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();
    // let model_entity = ecs.world.create_entity().with(obj_model).build();

    // let mut rng = rand::thread_rng();
    // let light_count = 5;
    // for _ in 0..light_count {
    //     ecs.world
    //         .create_entity()
    //         .with(SpotLight::new(
    //             glm::vec3(rng.gen_range(-50.0..50.0), 10.0, rng.gen_range(-50.0..50.0)),
    //             glm::Mat4::identity(),
    //             wgpu::Color {
    //                 a: 1.0,
    //                 b: rng.gen::<f64>(),
    //                 r: rng.gen::<f64>(),
    //                 g: rng.gen::<f64>(),
    //             },
    //             1.0,
    //             1.0,
    //             0.2,
    //             20.0,
    //             40.0,
    //         ))
    //         .build();
    //     ecs.world
    //         .create_entity()
    //         .with(PointLight::new(
    //             glm::vec3(
    //                 rng.gen_range(-100.0..100.0),
    //                 2.0,
    //                 rng.gen_range(-100.0..100.0),
    //             ),
    //             wgpu::Color {
    //                 a: 1.0,
    //                 b: rng.gen_range(0.0..1.0),
    //                 r: rng.gen_range(0.0..1.0),
    //                 g: rng.gen_range(0.0..1.0),
    //             },
    //             1.0,
    //             0.1,
    //             0.01,
    //         ))
    //         .build();
    // }
    // log::info!(" light count: {} ", light_count);

    // // INSTANCING
    // let mut physicsworld = ecs.world.write_resource::<PhysicsWorld>();
    // const SPACE: f32 = 10.0;
    // let mut instances = (0..NUM_INSTANCES_PER_ROW)
    //     .flat_map(|z| {
    //         (0..NUM_INSTANCES_PER_ROW).map(move |x| {
    //             let x = SPACE * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 1.5);
    //             let z = SPACE * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 1.5);
    //             let pos = glm::Vec3::new(x as f32, 10.0, z as f32);
    //             let rot = if pos == glm::vec3(0.0, 0.0, 0.0) {
    //                 glm::quat_angle_axis(f32::to_radians(0.0), &glm::vec3(0.0, 0.0, 1.0))
    //             } else {
    //                 glm::quat_angle_axis(f32::to_radians(45.0), &pos.clone().normalize())
    //             };
    //             Transform::new(pos, rot, glm::vec3(1.0, 1.0, 1.0), Some(model_entity))
    //         })
    //     })
    //     .collect::<Vec<_>>();
    // let physics_handles = instances
    //     .iter()
    //     .map(|instance| {
    //         let axisangle = if instance.position == glm::vec3(0.0, 0.0, 0.0) {
    //             f32::to_radians(0.0) * glm::vec3(0.0, 0.0, 1.0)
    //         } else {
    //             f32::to_radians(45.0) * instance.position.clone().normalize()
    //         };
    //         let rigid_body = RigidBodyBuilder::new_dynamic()
    //             .position(Isometry3::new(
    //                 Vec3::new(
    //                     instance.position.x,
    //                     instance.position.y,
    //                     instance.position.z,
    //                 ),
    //                 axisangle,
    //             ))
    //             .mass(1.0)
    //             .build();
    //         let rigid_body_handle = physicsworld.add_rigid_body(rigid_body);

    //         let collider = collision_builder.build();
    //         let collider_handle = physicsworld.add_collider(collider, rigid_body_handle);

    //         PhysicsHandle {
    //             collider_handle,
    //             rigid_body_handle,
    //         }
    //     })
    //     .collect::<Vec<_>>();

    // // ground object
    // let plane = Transform::new(
    //     glm::vec3(0.0, 0.5, 0.0),
    //     glm::quat_angle_axis(f32::to_radians(0.0), &glm::vec3(0.0, 0.0, 1.0)),
    //     glm::vec3(100.0, 1.0, 100.0),
    //     Some(model_entity),
    // );
    // instances.push(plane);
    // // ground shape
    // let ground_shape = ColliderBuilder::cuboid(100.0, 1.0, 100.0).build();

    // let ground_handle = physicsworld.add_rigid_body(
    //     RigidBodyBuilder::new_static()
    //         .position(Isometry3::new(
    //             glm::vec3(0.0, 0.5, 0.0),
    //             glm::vec3(0.0, 0.0, 1.0) * f32::to_radians(0.0),
    //         ))
    //         .build(),
    // );
    // let ground_collider = physicsworld.add_collider(ground_shape, ground_handle);

    // drop(physicsworld); // have to drop this to get world as mutable
    //                     // create the entities themselves.
    // ecs.world
    //     .create_entity()
    //     .with(plane)
    //     .with(PhysicsHandle {
    //         collider_handle: ground_collider,
    //         rigid_body_handle: ground_handle,
    //     })
    //     .build();
    // for (transform, physics_handle) in instances.iter().zip(physics_handles.iter()) {
    //     ecs.world
    //         .create_entity()
    //         .with(*transform)
    //         .with(*physics_handle)
    //         .build();
    // }

    // let mut storage = ecs.world.write_storage::<ModelCollider>();
    // storage
    //     .insert(model_entity, ModelCollider(collision_builder))
    //     .unwrap();
}
