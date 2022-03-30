use chrono::Duration;

pub struct DeltaTime {
    pub frame_count: u16,
    pub previous_frame_time: chrono::Duration,
    pub total_frame_time: chrono::Duration,
    /// Time between the last frame render in seconds
    pub delta: f32,
    pub app_start_time: i64,
    pub prev_sec_frame_count: u16,
}
impl Default for DeltaTime {
    fn default() -> Self {
        Self {
            frame_count: 0,
            previous_frame_time: Duration::nanoseconds(
                chrono::offset::Utc::now().timestamp_nanos(),
            ),
            total_frame_time: Duration::seconds(0),
            delta: 0.0,
            app_start_time: chrono::offset::Utc::now().timestamp_millis(),
            prev_sec_frame_count: 0,
        }
    }
}
