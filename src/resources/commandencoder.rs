pub struct HorizonCommandEncoder {
    cmd_encoder: wgpu::CommandEncoder,
}

impl HorizonCommandEncoder {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self {
            cmd_encoder: encoder,
        }
    }
    pub fn finish(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let command = std::mem::replace(
            &mut self.cmd_encoder,
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command encoder"),
            }),
        );
        let finished = command.finish();
        queue.submit(std::iter::once(finished));
    }
    pub fn get_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.cmd_encoder
    }
}
