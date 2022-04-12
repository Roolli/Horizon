use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::gpuquerysets::{
    GpuQuerySet, GpuQuerySetContainer, PipelineStatisticsQueries, TimestampQueries,
};
use crate::ui::gpustats::Passes;
use crate::ui::scriptingconsole::ScriptingConsole;
use crate::State;
use specs::{ReadExpect, System, Write, WriteExpect};

pub struct ResolveQuerySets;
impl ResolveQuerySets {
    pub const STAT_TYPES: [&'static str; 5] = [
        "vertex invocation",
        "clipper invocations",
        " clipper primitives count",
        "fragment shader invocation count",
        "compute shader invocation count",
    ];
}
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
        let timestamp_queries: &TimestampQueries = bytemuck::from_bytes(&*timestamp_view);

        let pipeline_stats_view = query_set
            .query_buffer
            .slice(GpuQuerySet::pipeline_statistics_offset()..)
            .get_mapped_range();
        let pipeline_stats_data: &PipelineStatisticsQueries =
            bytemuck::from_bytes(&*pipeline_stats_view);
        for (key, values) in &query_set.pass_indices {
            let pass_data = match key {
                Passes::ShadowPassWithCascade(i) => format!("Shadow cascade #{}", i),
                Passes::GBuffer => String::from("G Buffer "),
                Passes::LightCulling => String::from("Light culling"),
                Passes::Forward => String::from("Forward"),
                Passes::Collision => String::from("Collision"),
                Passes::Skybox => String::from("Skybox"),
                Passes::Ui => String::from("Ui"),
            };
            let timestamp_data = timestamp_queries[*values as usize];
            let nanos =
                (timestamp_data.end - timestamp_data.begin) as f32 * query_set.timestamp_period;
            let micros = nanos / 1000.0;
            let mut output_str = format!(
                "pass: #{} took {:.3} μs  \n pipeline_stats: \n",
                pass_data, micros
            );
            for (index, stat_type) in ResolveQuerySets::STAT_TYPES.into_iter().enumerate() {
                if pipeline_stats_data[*values as usize][index] > 0 {
                    output_str.push_str(
                        format!(
                            "{}: {}\n",
                            stat_type, pipeline_stats_data[*values as usize][index]
                        )
                        .as_str(),
                    );
                }
            }
            output_str.push('\n');
            scripting_console.messages.push(output_str);
        }
        query_set.pass_indices.clear();
        drop(pipeline_stats_view);
        query_set.next_query_index = 0;
        drop(timestamp_view);
        query_set.query_buffer.unmap();
    }
}
