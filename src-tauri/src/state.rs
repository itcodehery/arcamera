use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;
use crate::video::VideoRecorder;

pub struct AppState {
    pub is_recording: AtomicBool,
    pub recording_start: Mutex<Option<Instant>>,
    pub video_recorder: Mutex<Option<VideoRecorder>>,
}
