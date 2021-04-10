use core::panic;

use components::{
    physicshandle::PhysicsHandle,
    transform::{Transform, TransformRaw},
};
use filesystem::modelimporter::Importer;
//use wasm_bindgen::prelude::*;
use futures::executor::block_on;
use nalgebra::Isometry3;
use rapier3d::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder};
use renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::{
        lighting::LightBindGroup, shadow::ShadowBindGroup, uniforms::UniformBindGroup,
        HorizonBindGroup,
    },
    model::HorizonModel,
    modelbuilder::ModelBuilder,
    pipelines::{
        forwardpipeline::ForwardPipeline, lightpipeline::LightPipeline,
        shadowpipeline::ShadowPipeline, HorizonPipeline,
    },
    primitives::{
        lights::{
            directionallight::{DirectionalLight, DirectionalLightRaw},
            pointlight::{PointLight, PointLightRaw},
            spotlight::{SpotLight, SpotLightRaw},
        },
        uniforms::Globals,
    },
};
use resources::{
    bindingresourcecontainer::BindingResourceContainer, camera::Camera,
    commandencoder::HorizonCommandEncoder, windowevents::ResizeEvent,
};
use scripting::scriptingengine::V8ScriptingEngine;
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use systems::{
    physics::{Physics, PhysicsWorld},
    renderforwardpass::RenderForwardPass,
    rendershadowpass::RenderShadowPass,
    resize::Resize,
    updatebuffers::UpdateBuffers,
};

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
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
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
        let doc = win.document().unwrap();
        let body = doc.body().unwrap();
        body.append_child(&web_sys::Element::from(window.canvas()))
            .ok();
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_bindgen_futures::spawn_local(async move {
            let state = State::new(&window).await;
            let mut world = setup_ecs(state);
            create_debug_scene(&mut world.0).await;
            run(event_loop, window, world);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let state = block_on(State::new(&window));
        let mut ecs = setup_ecs(state);
        // ! for now block
        block_on(create_debug_scene(&mut ecs.0));
        run(event_loop, window, ecs);
    }
}
fn run(
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    mut ecs: (specs::World, specs::Dispatcher<'static, 'static>),
) {
    log::info!("running event loop");

    event_loop.run(move |event, _, control_flow| {
        // take ownership of ecs
        let _ = &ecs;
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    //TODO: add keyboard and mouse resources
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input
                        {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        let mut resize_event = ecs.0.write_resource::<ResizeEvent>();
                        resize_event.new_size = *physical_size;
                        resize_event.handled = false;
                        //state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        //state.resize(**new_inner_size);
                        let mut resize_event = ecs.0.write_resource::<ResizeEvent>();
                        resize_event.new_size = **new_inner_size;
                        resize_event.handled = false;
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                ecs.1.dispatch(&ecs.0);
                //TODO: handle events related to wgpu errors
                // Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                // Err(e) => log::error!("{:?}", e),
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
/// Initializes a new ECS container (World) and registers the components, creates the dependency chain for the system and sets up resources.
fn setup_ecs<'a, 'b>(state: State) -> (World, Dispatcher<'a, 'b>) {
    let mut world = World::new();
    world.insert(state);

    register_resources(&mut world);
    register_components(&mut world);
    // TODO: setup dispatcher

    let mut dispatcher = DispatcherBuilder::new()
        .with(Physics, stringify!(Physics), &[])
        .with_thread_local(Resize)
        .with_thread_local(UpdateBuffers)
        .with_thread_local(RenderShadowPass)
        .with_thread_local(RenderForwardPass)
        .build();
    dispatcher.setup(&mut world);

    (world, dispatcher)
}
fn register_resources(world: &mut World) {
    let state = world.read_resource::<State>();
    let size = state.size;
    let encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        });
    let importer = Importer::default();
    let model_builder = ModelBuilder::new(&state.device, importer);
    drop(state);
    world.insert(model_builder);
    world.insert(ResizeEvent {
        new_size: size,
        handled: false,
    });
    world.insert(PhysicsWorld::new(glm::Vec3::y() * -9.81));
    world.insert(BindingResourceContainer::default());

    world.insert(HorizonCommandEncoder::new(encoder));
}
fn register_components(mut world: &mut World) {
    world.register::<BindGroupContainer>();
    world.register::<ShadowBindGroup>();
    world.register::<UniformBindGroup>();
    world.register::<LightBindGroup>();
    setup_pipelines(&mut world);
}

fn setup_pipelines(world: &mut World) {
    let state = world.read_resource::<State>();
    let mut binding_resource_container = world.write_resource::<BindingResourceContainer>();
    UniformBindGroup::get_binding_resources(&state.device, &mut binding_resource_container);
    ShadowBindGroup::get_binding_resources(&state.device, &mut binding_resource_container);
    LightBindGroup::get_binding_resources(&state.device, &mut binding_resource_container);

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

    let forward_pipeline = ForwardPipeline::create_pipeline(
        &state.device,
        &state.sc_descriptor,
        (
            &world
                .read_resource::<ModelBuilder>()
                .diffuse_texture_bind_group_layout,
            &uniform_container.layout,
            &light_container.layout,
        ),
    );

    let shadow_pipeline = ShadowPipeline::create_pipeline(
        &state.device,
        &state.sc_descriptor,
        &shadow_container.layout,
    );

    let light_pipeline = LightPipeline::create_pipeline(
        &state.device,
        &state.sc_descriptor,
        (&uniform_container.layout, &light_container.layout),
    );

    drop(state);
    drop(binding_resource_container);
    world.insert(LightPipeline(light_pipeline));
    world.insert(ForwardPipeline(forward_pipeline));
    world.insert(ShadowPipeline(shadow_pipeline));
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
}

async fn create_debug_scene(world: &mut World) {
    let mut js = V8ScriptingEngine::new();
    js.execute(
        "test.js",
        r#"
        Horizon.print("it works!");
    "#,
    );

    const NUM_INSTANCES_PER_ROW: u32 = 15;
    world.insert(DirectionalLight::new(
        glm::vec3(1.0, -1.0, 0.0),
        wgpu::Color {
            r: 1.0,
            g: 0.5,
            b: 1.0,
            a: 1.0,
        },
    ));

    let state = world.write_resource::<State>();
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

    let obj_model = world
        .read_resource::<ModelBuilder>()
        .create(&state.device, &state.queue, "cube.obj")
        .await;
    drop(state);
    let collision_builder =
        ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();

    let mut globals = Globals::new(0, 0);
    globals.update_view_proj_matrix(&cam);
    let model_entity = world.create_entity().with(obj_model).build();
    world
        .create_entity()
        .with(SpotLight::new(
            glm::vec3(0.0, 0.0, 0.0),
            glm::Mat4::identity(),
            wgpu::Color::BLUE,
            1.0,
            0.4,
            0.6,
            20.0,
            40.0,
        ))
        .build();
    world
        .create_entity()
        .with(PointLight::new(
            glm::vec3(0.0, 0.0, 0.0),
            wgpu::Color::BLUE,
            1.0,
            0.4,
            0.6,
        ))
        .build();
    world.insert(globals);
    world.insert(cam);
    // INSTANCING
    let mut physicsworld = world.write_resource::<PhysicsWorld>();
    const SPACE: f32 = 10.0;
    let mut instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 1.5);
                let z = SPACE * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 1.5);
                let pos = glm::Vec3::new(x as f32, 10.0, z as f32);
                let rot = if pos == glm::vec3(0.0, 0.0, 0.0) {
                    glm::quat_angle_axis(f32::to_radians(0.0), &glm::vec3(0.0, 0.0, 1.0))
                } else {
                    glm::quat_angle_axis(f32::to_radians(45.0), &pos.clone().normalize())
                };
                Transform::new(pos, rot, glm::vec3(1.0, 1.0, 1.0), model_entity)
            })
        })
        .collect::<Vec<_>>();
    let physics_handles = instances
        .iter()
        .map(|instance| {
            let axisangle = if instance.position == glm::vec3(0.0, 0.0, 0.0) {
                f32::to_radians(0.0) * glm::vec3(0.0, 0.0, 1.0)
            } else {
                f32::to_radians(45.0) * instance.position.clone().normalize()
            };
            let rigid_body = RigidBodyBuilder::new_dynamic()
                .position(Isometry3::new(instance.position, axisangle))
                .mass(1.0)
                .build();
            let rigid_body_handle = physicsworld.add_rigid_body(rigid_body);

            let collider = collision_builder.build();
            let collider_handle = physicsworld.add_collider(collider, rigid_body_handle);

            PhysicsHandle {
                collider_handle,
                rigid_body_handle,
            }
        })
        .collect::<Vec<_>>();

    // ground object
    let plane = Transform::new(
        glm::vec3(0.0, 0.5, 0.0),
        glm::quat_angle_axis(f32::to_radians(0.0), &glm::vec3(0.0, 0.0, 1.0)),
        glm::vec3(100.0, 1.0, 100.0),
        model_entity,
    );
    instances.push(plane);
    // ground shape
    let ground_shape = ColliderBuilder::cuboid(100.0, 1.0, 100.0).build();

    let ground_handle = physicsworld.add_rigid_body(
        RigidBodyBuilder::new_static()
            .position(Isometry3::new(
                glm::vec3(0.0, 0.5, 0.0),
                glm::vec3(0.0, 0.0, 1.0) * f32::to_radians(0.0),
            ))
            .build(),
    );
    let ground_collider = physicsworld.add_collider(ground_shape, ground_handle);

    drop(physicsworld); // have to drop this to get world as mutable
                        // create the entities themselves.
    world
        .create_entity()
        .with(plane)
        .with(PhysicsHandle {
            collider_handle: ground_collider,
            rigid_body_handle: ground_handle,
        })
        .build();
    for (transform, physics_handle) in instances.iter().zip(physics_handles.iter()) {
        world
            .create_entity()
            .with(*transform)
            .with(*physics_handle)
            .build();
    }
}
