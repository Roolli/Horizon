use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::gpuquerysets::{
    GpuQuerySet, GpuQuerySetContainer, PipelineStatisticsQueries, TimestampQueries,
};
use crate::ui::scriptingconsole::ScriptingConsole;
use crate::State;
use specs::{ReadExpect, System, Write, WriteExpect};
use std::fmt::format;
use wgpu::Maintain;

pub struct ResolveQuerySets;

impl<'a> System<'a> for ResolveQuerySets {
    type SystemData = (
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, State>,
        WriteExpect<'a, GpuQuerySetContainer>,
        Write<'a, ScriptingConsole>,
    );

    fn run(
        &mut self,
        (mut command_encoder, state, mut query_sets, mut scripting_console): Self::SystemData,
    ) {
        if query_sets.container.is_none() {
            return;
        }
        let query_set = query_sets.container.as_mut().unwrap();
        let num_passes = (State::NUM_PASSES + State::SHADOW_SIZE.depth_or_array_layers);
        let timestamp_query_count = query_set.next_query_index * 2;
        let cmd_encoder = command_encoder.get_encoder();
        cmd_encoder.resolve_query_set(
            &query_set.timestamp_queries,
            0..timestamp_query_count,
            &query_set.query_buffer,
            0,
        );
        cmd_encoder.resolve_query_set(
            &query_set.pipeline_queries,
            0..query_set.next_query_index,
            &query_set.query_buffer,
            GpuQuerySet::pipeline_statistics_offset(),
        );
        command_encoder.finish(&state.device, &state.queue);
        let _ = query_set
            .query_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read);
        state.device.poll(wgpu::Maintain::Wait);
        let timestamp_view = query_set
            .query_buffer
            .slice(..std::mem::size_of::<TimestampQueries>() as wgpu::BufferAddress)
            .get_mapped_range();
        let timestamp_data: &TimestampQueries = bytemuck::from_bytes(&*timestamp_view);
        // for (index, data) in timestamp_data.iter().enumerate() {
        //     let nanos = (data.end - data.begin) as f32 * query_set.timestamp_period;
        //     let micros = nanos / 1000.0;
        //     log::info!("pass #{} took {:.3} μs ", index, micros);
        // }
        let pipeline_stats_view = query_set
            .query_buffer
            .slice(GpuQuerySet::pipeline_statistics_offset()..)
            .get_mapped_range();
        let pipeline_stats_data: &PipelineStatisticsQueries =
            bytemuck::from_bytes(&*pipeline_stats_view);
        for (index, (timestamp_data, query_stats)) in timestamp_data
            .iter()
            .zip(pipeline_stats_data.iter())
            .enumerate()
        {
            log::info!(
                r#"pass #{} pipeline_stats: \n
            vertex invocation:{}\n
            clipper invocations:{}\n
            clipper primitives count:{}\n
            fragment shader invocation count:{}\n
            compute shader invocation count:{}
             "#,
                index,
                query_stats[0],
                query_stats[1],
                query_stats[2],
                query_stats[3],
                query_stats[4]
            )
        }
        drop(pipeline_stats_view);
        query_set.next_query_index = 0;
        drop(timestamp_view);
        query_set.query_buffer.unmap();
    }
}
