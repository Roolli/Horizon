use chrono::Duration;

pub struct DeltaTime {
   pub frame_count: u32,
   pub previous_frame_time: chrono::Duration,
   pub total_frame_time: chrono::Duration,
}
impl Default for DeltaTime {
    fn default() -> Self {
        Self {
            frame_count: 0,
            previous_frame_time: Duration::nanoseconds(
                chrono::offset::Utc::now().timestamp_nanos(),
            ),
            total_frame_time: Duration::seconds(0),
        }
    }
}
