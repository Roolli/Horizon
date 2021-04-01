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
        let mut swapped_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command encoder"),
        });
        std::mem::swap(&mut self.cmd_encoder, &mut swapped_encoder);
        let command = swapped_encoder.finish();
        queue.submit(std::iter::once(command));
    }
    pub fn get_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.cmd_encoder
    }
}
