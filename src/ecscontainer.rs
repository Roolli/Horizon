use std::borrow::{Borrow, BorrowMut};
use js_sys::JSON::stringify;
use rapier3d::na::Vector3;
use ref_thread_local::{Ref, RefMut, RefThreadLocal};
use specs::{DispatcherBuilder, World, WorldExt};

use crate::components::assetidentifier::AssetIdentifier;
use crate::components::modelcollider::ModelCollider;
use crate::scripting::scriptevent::ScriptEvent;
use crate::{components::scriptingcallback::ScriptingCallback, ECS_CONTAINER, filesystem::modelimporter::Importer, renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::{
        deferred::DeferredBindGroup, lighting::LightBindGroup, shadow::ShadowBindGroup,
        tiling::TilingBindGroup, uniforms::UniformBindGroup,
    },
    modelbuilder::ModelBuilder,
    state::State,
}, resources::{
    bindingresourcecontainer::BindingResourceContainer,
    commandencoder::HorizonCommandEncoder,
    eguirenderpass::EguiRenderPass,
    windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent, ResizeEvent},
}, systems::{
    physics::{Physics, PhysicsWorld},
}};
use crate::systems::events::handlewindowevents::HandleWindowEvents;
use crate::systems::events::resize::Resize;
use crate::systems::rendering::computelightculling::ComputeLightCulling;
use crate::systems::rendering::renderforwardpass::RenderForwardPass;
use crate::systems::rendering::rendershadowpass::RenderShadowPass;
use crate::systems::rendering::updatebuffers::UpdateBuffers;
use crate::systems::rendering::updatecamera::UpdateCamera;
use crate::systems::rendering::writegbuffer::WriteGBuffer;
use crate::systems::util::calculatedeltatime::UpdateDeltaTime;

pub struct ECSContainer {
    pub world: specs::World,
    pub dispatcher: specs::Dispatcher<'static, 'static>,
}

impl ECSContainer {
    pub fn dispatch(&mut self)
    {
        self.dispatcher.dispatch(&self.world);
    }
    pub fn new() -> Self {
        let mut world = World::new();

        // TODO: setup dispatcher

        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdateDeltaTime,stringify!(UpdateDeltaTime),&[])
            .with(UpdateCamera,stringify!(UpdateCamera),&[])
            .with(HandleWindowEvents,stringify!(HandleWindowEvents),&[])
            .with(Physics, stringify!(Physics), &[])
            .with_thread_local(Resize)
            .with_thread_local(UpdateBuffers)
            .with_thread_local(RenderShadowPass)
            .with_thread_local(WriteGBuffer)
            .with_thread_local(ComputeLightCulling)
            .with_thread_local(RenderForwardPass)
            .build();
        dispatcher.setup(&mut world);
        ECSContainer::register_components(&mut world);
        Self { dispatcher, world }
    }
    pub fn setup(&mut self, state: State) {
        self.world.insert(state);

        ECSContainer::register_resources(&mut self.world);
    }
    fn register_resources(world: &mut specs::World) {
        let state = world.read_resource::<State>();
        let size = state.size;
        let encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        let importer = Importer::default();
        let model_builder = ModelBuilder::new(&state.device, importer);
        let egui_render_pass = EguiRenderPass {
            pass: egui_wgpu_backend::RenderPass::new(&state.device, state.sc_descriptor.format, 1),
        };

        drop(state);
        world.insert(model_builder);
        world.insert(egui_render_pass);
        world.insert(ResizeEvent {
            new_size: size,
            handled: false,
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
        world.register::<ScriptingCallback>();
        world.register::<ScriptEvent>();
        world.register::<AssetIdentifier>();
    }
    pub fn global<'a>() -> Ref<'a,ECSContainer>  {
         ref_thread_local::RefThreadLocal::borrow(&ECS_CONTAINER)
    }
    pub fn global_mut<'a>() -> RefMut<'a,ECSContainer> {
        ref_thread_local::RefThreadLocal::borrow_mut(&ECS_CONTAINER)
    }
}
