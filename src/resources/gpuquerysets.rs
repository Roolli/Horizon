use crate::State;
use bytemuck::*;

pub struct GpuQuerySetContainer {
    pub container: Option<GpuQuerySet>,
}
pub struct GpuQuerySet {
    pub timestamp_queries: wgpu::QuerySet,
    pub pipeline_queries: wgpu::QuerySet,
    pub query_buffer: wgpu::Buffer,
    pub timestamp_period: f32,
    pub next_query_index: u32,
}
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TimestampData {
    pub begin: u64,
    pub end: u64,
}
pub type TimestampQueries =
    [TimestampData; (State::NUM_PASSES + State::SHADOW_SIZE.depth_or_array_layers) as usize];
pub type PipelineStatisticsQueries =
    [[u64; 5]; (State::NUM_PASSES + State::SHADOW_SIZE.depth_or_array_layers) as usize];
impl GpuQuerySet {
    pub fn pipeline_statistics_offset() -> wgpu::BufferAddress {
        (std::mem::size_of::<TimestampQueries>() as wgpu::BufferAddress)
            .max(wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT)
    }
}
