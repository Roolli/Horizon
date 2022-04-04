use deno_core::{v8, JsRuntime, RuntimeOptions};
use rapier3d::na::{Point3, Vector3};
use ref_thread_local::{Ref, RefMut, RefThreadLocal};
use specs::{DispatcherBuilder, RunNow, System, World, WorldExt};
use std::borrow::{Borrow, BorrowMut};

use crate::components::assetidentifier::AssetIdentifier;
use crate::components::modelcollider::ModelCollider;
use crate::resources::scriptingstate::ScriptingState;
use crate::resources::surfacetexture::SurfaceTexture;
use crate::scripting::scriptevent::ScriptEvent;
use crate::scripting::scriptingengine::HorizonScriptingEngine;
use crate::{
    components::scriptingcallback::ScriptingCallback,
    filesystem::modelimporter::Importer,
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{
            deferred::DeferredBindGroup, lighting::LightBindGroup, shadow::ShadowBindGroup,
            tiling::TilingBindGroup, uniforms::UniformBindGroup,
        },
        modelbuilder::ModelBuilder,
        state::State,
    },
    resources::{
        bindingresourcecontainer::BindingResourceContainer,
        commandencoder::HorizonCommandEncoder,
        windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent, ResizeEvent},
    },
    systems::physics::{Physics, PhysicsWorld},
    Camera, CameraController, DebugCollisionBindGroup, DebugTextureBindGroup,
    DefaultTextureContainer, DirectionalLight, Globals, HorizonModel, Projection, RawModel,
    SkyboxBindGroup, TextureViewTypes, WindowState, ECS_CONTAINER,
};

use crate::systems::events::handlewindowevents::HandleInternalWindowEvents;
use crate::systems::events::resize::Resize;
use crate::systems::rendering::acquiretexture::AcquireTexture;
use crate::systems::rendering::computelightculling::ComputeLightCulling;
use crate::systems::rendering::rendercollision::RenderCollision;
use crate::systems::rendering::renderforwardpass::RenderForwardPass;
use crate::systems::rendering::rendershadowpass::RenderShadowPass;
use crate::systems::rendering::renderskybox::RenderSkyBox;
use crate::systems::rendering::renderuipass::RenderUIPass;
use crate::systems::rendering::updatebuffers::UpdateBuffers;
use crate::systems::rendering::updatecamera::UpdateCamera;
use crate::systems::rendering::writegbuffer::WriteGBuffer;
use crate::systems::util::calculatedeltatime::UpdateDeltaTime;
use crate::ui::debugstats::DebugStats;

pub struct ECSContainer {
    pub world: specs::World,
    pub dispatcher: specs::Dispatcher<'static, 'static>,
}
impl Default for ECSContainer {
    fn default() -> Self {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdateDeltaTime, stringify!(UpdateDeltaTime), &[])
            .with(
                HandleInternalWindowEvents,
                stringify!(HandleWindowEvents),
                &[],
            )
            .with(UpdateCamera, stringify!(UpdateCamera), &[])
            .with(Physics, stringify!(Physics), &[])
            .with_thread_local(Resize)
            .with_thread_local(UpdateBuffers)
            .with_thread_local(RenderShadowPass)
            .with_thread_local(WriteGBuffer)
            .with_thread_local(ComputeLightCulling)
            .with_thread_local(AcquireTexture)
            .with_thread_local(RenderForwardPass)
            .with_thread_local(RenderCollision)
            .with_thread_local(RenderSkyBox)
            .with_thread_local(RenderUIPass)
            .build();
        dispatcher.setup(&mut world);
        ECSContainer::register_components(&mut world);
        Self { dispatcher, world }
    }
}
impl ECSContainer {
    pub fn dispatch(&mut self) {
        self.dispatcher.dispatch(&self.world);
    }
    pub fn setup(&mut self, state: State) {
        self.world.insert(state);
        Self::register_resources(&mut self.world);
        self.initialize_scripting();
    }
    fn initialize_scripting(&mut self) {
        self.world.insert(HorizonScriptingEngine::default());
    }

    fn register_resources(world: &mut specs::World) {
        let state = world.read_resource::<State>();
        let size = state.size;
        let encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        let egui_render_pass =
            egui_wgpu_backend::RenderPass::new(&state.device, state.sc_descriptor.format, 1);
        let default_texture_container =
            DefaultTextureContainer::create(&state.device, &state.queue);
        let mut globals = Globals::new(0, 0);
        let cam = Camera::new(
            Point3::new(-2.0, 1.9, 0.5),
            f32::to_radians(-2.0),
            f32::to_radians(-16.0),
        );
        let proj = Projection::new(
            state.sc_descriptor.width,
            state.sc_descriptor.height,
            f32::to_radians(45.0),
            0.01,
        );
        let cam_controller = CameraController::new(10.0, 2.0);
        globals.update_view_proj_matrix(&cam, &proj);
        drop(state);

        world.insert(cam_controller);
        world.insert(proj);
        world.insert(globals);
        world.insert(cam);
        world.insert(DirectionalLight::new(
            0.0,
            45.0_f32.to_radians(),
            wgpu::Color {
                r: 0.9,
                g: 0.7,
                b: 0.5,
                a: 1.0,
            },
        ));
        world.insert(default_texture_container);
        world.insert(egui_render_pass);
        world.insert(ResizeEvent {
            new_size: size,
            handled: false,
            scale_factor: None,
        });
        world.insert(ScriptingState {
            run_physics_simulation: true,
            show_collision_meshes: false,
        });
        world.insert(SurfaceTexture { texture: None });
        world.insert(DebugStats {
            fps: 0,
            unique_model_count: 1,
            messages: Vec::new(),
            debug_texture: None,
            debug_texture_view: None,
            cam_pos: rapier3d::na::Point3::new(0.0, 0.0, 0.0),
            cam_yaw_pitch: (0.0, 0.0),
            texture_id: None,
            selected_texture_name: TextureViewTypes::DeferredPosition,
            debug_texture_renderer: None,
            selected_entity: None,
            selected_texture: 0,
            selected_material: 0,
        });
        world.insert(WindowState {
            cursor_state: false,
        });
        world.insert(KeyboardEvent::default());
        world.insert(MouseMoveEvent::default());
        world.insert(MouseInputEvent::default());
        world.insert(PhysicsWorld::new(Vector3::y() * -9.81));
        world.insert(BindingResourceContainer::default());
        world.insert(HorizonCommandEncoder::new(encoder));
    }

    fn register_components(world: &mut World) {
        world.register::<ModelCollider>();
        world.register::<BindGroupContainer>();
        world.register::<ShadowBindGroup>();
        world.register::<UniformBindGroup>();
        world.register::<LightBindGroup>();
        world.register::<DeferredBindGroup>();
        world.register::<TilingBindGroup>();
        world.register::<SkyboxBindGroup>();
        world.register::<DebugTextureBindGroup>();
        world.register::<DebugCollisionBindGroup>();
        world.register::<ScriptingCallback>();
        world.register::<ScriptEvent>();
        world.register::<AssetIdentifier>();
        world.register::<RawModel>();
        world.register::<HorizonModel>();
    }
    pub fn global<'a>() -> Ref<'a, ECSContainer> {
        ref_thread_local::RefThreadLocal::borrow(&ECS_CONTAINER)
    }
    pub fn global_mut<'a>() -> RefMut<'a, ECSContainer> {
        ref_thread_local::RefThreadLocal::borrow_mut(&ECS_CONTAINER)
    }
}
#[derive(Debug, Clone)]
pub enum ECSError {
    EntityNotFound,
    InvalidComponentData(String),
}
