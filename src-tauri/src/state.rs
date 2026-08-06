use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;

pub struct AppState {
    pub is_recording: AtomicBool,
    pub recorded_frames: Mutex<Vec<Vec<u8>>>,
    pub recording_start: Mutex<Option<Instant>>,
}
