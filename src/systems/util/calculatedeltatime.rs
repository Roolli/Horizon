use std::ops::Add;
use chrono::{Duration, Timelike};
use specs::{System, Write, WriteExpect};
use crate::resources::deltatime::DeltaTime;
use crate::ui::debugstats::DebugStats;

pub struct UpdateDeltaTime;

impl<'a> System<'a> for UpdateDeltaTime {
    type SystemData = (Write<'a,DeltaTime>,WriteExpect<'a,DebugStats>);


    fn run(&mut self, (mut delta_time,mut debug_ui): Self::SystemData) {

        let now = chrono::offset::Utc::now();
        let delta = (now - delta_time.previous_frame_time).timestamp_nanos();
        delta_time.delta = delta as f32 / 1_000_000_000.0;
        delta_time.total_frame_time = delta_time.total_frame_time.add(Duration::nanoseconds((now - delta_time.previous_frame_time).timestamp_nanos()));

        if delta_time.total_frame_time < Duration::seconds(1) {
            delta_time.frame_count += 1;
        } else {
            delta_time.prev_sec_frame_count = delta_time.frame_count;
            debug_ui.fps = delta_time.prev_sec_frame_count;
            delta_time.frame_count = 0;
            delta_time.total_frame_time = Duration::seconds(0);
        }
        delta_time.previous_frame_time = Duration::nanoseconds(now.timestamp_nanos());
    }
}