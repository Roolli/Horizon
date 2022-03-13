use specs::{ReadExpect, System, Write, WriteExpect};
use crate::resources::surfacetexture::SurfaceTexture;
use crate::{RenderResult, State};

pub struct AcquireTexture;


impl<'a> System<'a> for AcquireTexture {
    type SystemData = (WriteExpect<'a,SurfaceTexture>,ReadExpect<'a,State>,Write<'a,RenderResult>);

    fn run(&mut self, (mut surface_texture,state,mut render_result): Self::SystemData) {
        let frame_result = state.surface.get_current_texture();
        let frame;
        if let Ok(f) = frame_result {
            frame = f;
        } else {
            render_result.result = frame_result.err();
            return;
        }
        surface_texture.texture = Some(frame);
    }
}