use specs::{ReadExpect, System, WriteExpect};

use crate::{renderer::state::State, resources::eguirenderpass::EguiRenderPass};

pub struct RenderUIPass;

impl<'a> System<'a> for RenderUIPass {
    type SystemData = (ReadExpect<'a, State>, WriteExpect<'a, EguiRenderPass>);

    fn run(&mut self, data: Self::SystemData) {}
}
