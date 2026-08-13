use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::watch::Sender;
use crate::video::VideoRecorder;

pub struct AppState {
    pub is_recording: AtomicBool,
    pub recording_start: Mutex<Option<Instant>>,
    pub video_recorder: Mutex<Option<VideoRecorder>>,
    pub frame_tx: Sender<Vec<u8>>,
    pub latest_frame: Mutex<Option<Vec<u8>>>,
}
