use core::panic;

use components::{physicshandle::PhysicsHandle, transform::Transform};
//use wasm_bindgen::prelude::*;
use futures::executor::block_on;
use specs::{DispatcherBuilder, World, WorldExt};
use systems::physics::{Physics, PhysicsWorld};

mod filesystem;
mod renderer;
mod resources;

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
            let mut state = State::new(&window).await;
            run(event_loop, state, window);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let state = block_on(State::new(&window));
        run(event_loop, state, window);
    }
}
fn run(event_loop: EventLoop<()>, mut state: State, window: winit::window::Window) {
    log::info!("running event loop");

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
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
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                Err(e) => log::error!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
/// Initializes a new ECS container (World) and registers the components and resources.
fn setup_ecs() -> World {
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(Physics, stringify!(Physics), &[])
        .build();
    dispatcher.setup(&mut world);
    world.insert(PhysicsWorld::new(glm::Vec3::y() * -9.81));
    register_components(&mut world);
    world
}
fn register_components(world: &mut World) {
    world.register::<Transform>();
    world.register::<PhysicsHandle>();
}
fn create_debug_scene() {
    // TODO: Change to some sort of IoC container where it resolves based on current arch.
    let importer = Importer::default();

    // CAMERA
    let cam = Camera {
        eye: glm::vec3(-10.0, 15.0, 10.0),
        target: glm::vec3(0.0, 0.0, 0.0),
        up: glm::vec3(0.0, 1.0, 0.0), // Unit Y vector
        aspect_ratio: sc_desc.width as f32 / sc_desc.height as f32,
        fov_y: 90.0,
        z_near: 0.1,
        z_far: 300.0,
    };

    // MODEL LOADING
    let model_builder = ModelBuilder::new(&device, importer);
    let obj_model = model_builder.create(&device, &queue, "cube.obj").await;
    let model_entity = world.create_entity().with(obj_model).build();

    // INSTANCING
    let mut physicsworld = world.write_resource::<PhysicsWorld>();
    const SPACE: f32 = 10.0;
    let mut collision_builder =
        ColliderBuilder::convex_hull(obj_model.meshes[0].points.as_slice()).unwrap();
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
                Transform::new(pos, rot, glm::vec3(1.0, 1.0, 1.0))
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
    physicsworld.add_collider(ground_shape, ground_handle);

    drop(physicsworld); // have to drop this to get world as mutable

    for (transform, physics_handle) in instances.iter().zip(physics_handles.iter()) {
        world
            .create_entity()
            .with(*transform)
            .with(*physics_handle)
            .build();
    }
    let instance_data = instances.iter().map(Transform::to_raw).collect::<Vec<_>>();
    let normal_matricies = instance_data
        .iter()
        .map(TransformRaw::get_normal_matrix)
        .collect::<Vec<_>>();

    let mut globals = Globals::new(lights.len() as u32);
    globals.update_view_proj_matrix(&cam);
}
