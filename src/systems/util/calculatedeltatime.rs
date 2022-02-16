use std::ops::Add;
use chrono::{Duration, Timelike};
use specs::{System, Write, WriteExpect};
use crate::resources::deltatime::DeltaTime;

pub struct UpdateDeltaTime;

impl<'a> System<'a> for UpdateDeltaTime {
    type SystemData = Write<'a,DeltaTime>;

    fn run(&mut self, mut delta_time: Self::SystemData) {

        let now = chrono::offset::Utc::now();
        let delta = (now - delta_time.previous_frame_time).timestamp_nanos();
        delta_time.delta = delta as f32 / 1_000_000_000.0;
        delta_time.total_frame_time = delta_time.total_frame_time.add(Duration::nanoseconds((now - delta_time.previous_frame_time).timestamp_nanos()));

        if delta_time.total_frame_time < Duration::seconds(1) {
            delta_time.frame_count += 1;
        } else {
            log::info!(target:"Frame Count","FPS: {}", delta_time.frame_count);
            delta_time.frame_count = 0;
            delta_time.total_frame_time = Duration::seconds(0);
        }
        delta_time.previous_frame_time = Duration::nanoseconds(now.timestamp_nanos());
    }
}