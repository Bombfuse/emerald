pub struct ProfileSettings {
    /// Number of frames to keep in memory for each profile, before dropping
    /// the oldest frame.
    pub frame_limit: usize,
}
impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            // Approximately 10 seconds worth of frame data.
            frame_limit: 600,
        }
    }
}
