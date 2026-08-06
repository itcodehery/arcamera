use std::sync::Arc;
use std::time::Instant;
use std::sync::atomic::Ordering;
use tauri::State;
use base64::{Engine as _, engine::general_purpose};
use std::path::PathBuf;
use std::fs;
use chrono::Local;

use crate::state::AppState;
use crate::video;
use crate::gallery;

#[tauri::command]
pub fn save_photo(data_url: String) -> Result<String, String> {
    let (data_to_decode, ext) = if data_url.starts_with("data:image/jpeg;base64,") {
        (&data_url["data:image/jpeg;base64,".len()..], "jpg")
    } else if data_url.starts_with("data:image/png;base64,") {
        (&data_url["data:image/png;base64,".len()..], "png")
    } else {
        return Err("Invalid image format".to_string());
    };

    let image_bytes = general_purpose::STANDARD
        .decode(data_to_decode)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

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
pub fn start_recording(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let recorder = video::VideoRecorder::start(30)?; // Target 30 fps
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
