pub struct HorizonCommandEncoder {
    pub cmd_encoder: wgpu::CommandEncoder,
}

impl HorizonCommandEncoder {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self {
            cmd_encoder: encoder,
        }
    }
}
