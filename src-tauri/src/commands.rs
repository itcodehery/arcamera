use std::sync::Arc;
use std::time::Instant;
use std::sync::atomic::Ordering;
use tauri::State;
use std::path::PathBuf;
use std::fs;
use chrono::Local;

use crate::state::AppState;
use crate::video;
use crate::gallery;

#[tauri::command]
pub fn save_photo(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let frame_opt = state.latest_frame.lock().unwrap().clone();
    let image_bytes = frame_opt.ok_or_else(|| "No camera frame available yet".to_string())?;
    let ext = "jpg";

    // Phase 1 cross-platform fix using dirs crate
    let pictures_dir = dirs::picture_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Arcamera");

    if !pictures_dir.exists() {
        fs::create_dir_all(&pictures_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("photo_{}.{}", timestamp, ext);
    let filepath = pictures_dir.join(filename);

    fs::write(&filepath, image_bytes)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(filepath.to_string_lossy().to_string())
}

#[tauri::command]
pub fn start_recording(app: tauri::AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let recorder = video::VideoRecorder::start(app, 30)?; // Target 30 fps
    *state.video_recorder.lock().unwrap() = Some(recorder);
    *state.recording_start.lock().unwrap() = Some(Instant::now());
    state.is_recording.store(true, Ordering::Release);
    Ok(())
}

#[tauri::command]
pub fn stop_recording(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    state.is_recording.store(false, Ordering::Release);
    state.recording_start.lock().unwrap().take();
    
    let recorder = state.video_recorder.lock().unwrap().take();
    if let Some(recorder) = recorder {
        recorder.stop()
    } else {
        Err("No recording in progress".to_string())
    }
}

#[tauri::command]
pub fn list_gallery_items() -> Result<Vec<gallery::GalleryItem>, String> {
    gallery::list_items()
}

#[tauri::command]
pub fn get_thumbnail(path: String) -> Result<String, String> {
    gallery::get_thumbnail(&path)
}

#[tauri::command]
pub fn delete_gallery_item(path: String) -> Result<(), String> {
    gallery::delete_item(&path)
}

#[tauri::command]
pub fn open_file_externally(path: String) -> Result<(), String> {
    gallery::open_externally(&path)
}
