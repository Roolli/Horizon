use std::fmt::Debug;

use once_cell::sync::OnceCell;
use rapier3d::geometry::ColliderBuilder;
use specs::{Component, DispatcherBuilder, World, WorldExt};

use crate::components::modelcollider::ModelCollider;
use crate::scripting::lifecycleevents::LifeCycleEvent;
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
        eguirenderpass::EguiRenderPass,
        windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent, ResizeEvent},
    },
    systems::{
        computelightculling::ComputeLightCulling,
        physics::{Physics, PhysicsWorld},
        renderforwardpass::RenderForwardPass,
        rendershadowpass::RenderShadowPass,
        resize::Resize,
        updatebuffers::UpdateBuffers,
        writegbuffer::WriteGBuffer,
    },
    ECS_INSTANCE,
};

pub struct ECSContainer {
    pub world: specs::World,
    pub dispatcher: specs::Dispatcher<'static, 'static>,
}
// impl Debug for ECSContainer {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ECSContainer").finish()
//     }
// }

impl ECSContainer {
    pub fn new() -> Self {
        let mut world = World::new();

        // TODO: setup dispatcher

        let mut dispatcher = DispatcherBuilder::new()
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
        world.insert(PhysicsWorld::new(glm::Vec3::y() * -9.81));
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
        world.register::<LifeCycleEvent>();
    }
    pub fn global() -> &'static ECSContainer {
        unsafe { ECS_INSTANCE.get().expect("ECS was not initialized") }
    }
    pub fn global_mut() -> &'static mut ECSContainer {
        unsafe { ECS_INSTANCE.get_mut().unwrap() }
    }
}
