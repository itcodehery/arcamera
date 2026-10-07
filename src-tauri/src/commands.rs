use std::sync::Arc;
use std::time::Instant;
use std::sync::atomic::Ordering;
use tauri::State;
use std::path::PathBuf;
use std::fs;
use std::io::Cursor;
use chrono::Local;

use crate::state::AppState;
use crate::video;
use crate::gallery;

#[tauri::command]
pub async fn save_photo(
    state: State<'_, Arc<AppState>>,
    format: Option<String>,
    mirror_output: Option<bool>,
    aspect_ratio: Option<String>,
    burst_count: Option<u32>,
) -> Result<Vec<String>, String> {
    let count = burst_count.unwrap_or(1).max(1);
    
    let mut frames = Vec::with_capacity(count as usize);
    
    if count == 1 {
        let frame_opt = state.latest_frame.lock().unwrap().clone();
        let image_bytes = frame_opt.ok_or_else(|| "No camera frame available yet".to_string())?;
        frames.push(image_bytes);
    } else {
        let mut rx = state.frame_tx.subscribe();
        for _ in 0..count {
            match rx.changed().await {
                Ok(_) => {
                    frames.push(rx.borrow().clone());
                }
                Err(_) => {
                    return Err("Camera stream disconnected during burst".to_string());
                }
            }
        }
    }

    let fmt = format.unwrap_or_else(|| "jpg".to_string()).to_lowercase();
    let ext = match fmt.as_str() {
        "png" => "png",
        _ => "jpg",
    };
    let should_mirror = mirror_output.unwrap_or(false);
    let ratio_str = aspect_ratio.unwrap_or_else(|| "native".to_string());

    tokio::task::spawn_blocking(move || {
        let pictures_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("Arcamera");

        if !pictures_dir.exists() {
            fs::create_dir_all(&pictures_dir)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let needs_processing = ext == "png" || should_mirror || ratio_str != "native";
        let mut saved_paths = Vec::new();

        for image_bytes in frames {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f");
            let filename = format!("photo_{}.{}", timestamp, ext);
            let filepath = pictures_dir.join(&filename);

            if needs_processing {
                let mut img = image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Jpeg)
                    .map_err(|e| format!("Failed to decode JPEG frame: {}", e))?;

                let (width, height) = (img.width(), img.height());
                let target_ratio = match ratio_str.as_str() {
                    "1:1" => Some(1.0f32),
                    "4:3" => Some(4.0 / 3.0),
                    "16:9" => Some(16.0 / 9.0),
                    _ => None,
                };

                if let Some(target) = target_ratio {
                    let current = width as f32 / height as f32;
                    let (crop_w, crop_h) = if current > target {
                        let w = (height as f32 * target).round() as u32;
                        (w, height)
                    } else {
                        let h = (width as f32 / target).round() as u32;
                        (width, h)
                    };
                    let offset_x = (width - crop_w) / 2;
                    let offset_y = (height - crop_h) / 2;
                    img = img.crop_imm(offset_x, offset_y, crop_w, crop_h);
                }
                    
                if should_mirror {
                    img = img.fliph();
                }
                    
                let mut buf = Vec::new();
                let mut cursor = Cursor::new(&mut buf);
                let output_format = if ext == "png" { image::ImageFormat::Png } else { image::ImageFormat::Jpeg };
                
                img.write_to(&mut cursor, output_format)
                    .map_err(|e| format!("Failed to encode image: {}", e))?;
                    
                fs::write(&filepath, buf)
                    .map_err(|e| format!("Failed to save image: {}", e))?;
            } else {
                fs::write(&filepath, image_bytes)
                    .map_err(|e| format!("Failed to save image: {}", e))?;
            }
            
            saved_paths.push(filepath.to_string_lossy().to_string());
        }

        Ok(saved_paths)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn start_recording(app: tauri::AppHandle, state: State<'_, Arc<AppState>>, target_fps: Option<u32>) -> Result<(), String> {
    let fps = target_fps.unwrap_or(30);
    let recorder = video::VideoRecorder::start(app, fps)?;
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
pub async fn get_thumbnail(app: tauri::AppHandle, path: String) -> Result<String, String> {
    gallery::get_thumbnail(&app, &path).await
}

#[tauri::command]
pub fn get_stream_port(state: State<'_, Arc<AppState>>) -> u16 {
    state.server_port.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn delete_gallery_item(path: String) -> Result<(), String> {
    gallery::delete_item(&path)
}

#[tauri::command]
pub fn open_file_externally(path: String) -> Result<(), String> {
    gallery::open_externally(&path)
}

#[tauri::command]
pub fn set_camera_fps(state: State<'_, Arc<AppState>>, fps: u32) {
    state.target_fps.store(fps, std::sync::atomic::Ordering::Relaxed);
}
