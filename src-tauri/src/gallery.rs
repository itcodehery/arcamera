use std::fs;
use std::path::PathBuf;
use std::io::Cursor;
use chrono::Local;
use base64::{Engine as _, engine::general_purpose};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[derive(serde::Serialize)]
pub struct GalleryItem {
    pub path: String,
    pub filename: String,
    pub is_video: bool,
    pub size_bytes: u64,
    pub modified: String,
}

pub fn list_items() -> Result<Vec<GalleryItem>, String> {
    let mut items = Vec::new();

    let pictures_dir = dirs::picture_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Arcamera");

    let videos_dir = dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Arcamera");

    let dirs = [pictures_dir, videos_dir];

    let image_exts = ["jpg", "jpeg", "png"];
    let video_exts = ["mp4", "webm", "mkv", "avi"];

    for dir in &dirs {
        if !dir.exists() {
            continue;
        }
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let is_image = image_exts.contains(&ext.as_str());
            let is_video = video_exts.contains(&ext.as_str());
            if !is_image && !is_video {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = metadata
                .modified()
                .ok()
                .map(|t| {
                    let dt: chrono::DateTime<Local> = t.into();
                    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
                })
                .unwrap_or_default();

            items.push(GalleryItem {
                path: path.to_string_lossy().to_string(),
                filename: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                is_video,
                size_bytes: metadata.len(),
                modified,
            });
        }
    }

    // Newest first
    items.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(items)
}

fn get_cache_path(source_path: &str) -> Option<PathBuf> {
    let p = PathBuf::from(source_path);
    let parent = p.parent()?;
    let stem = p.file_stem()?.to_str()?;
    let cache_dir = parent.join(".thumbnails");
    if !cache_dir.exists() {
        let _ = fs::create_dir_all(&cache_dir);
    }
    Some(cache_dir.join(format!("{}_thumb.jpg", stem)))
}

pub async fn get_thumbnail(app: &AppHandle, path: &str) -> Result<String, String> {
    let cache_path = get_cache_path(path);
    if let Some(ref thumb) = cache_path {
        if thumb.exists() {
            if let Ok(bytes) = fs::read(thumb) {
                return Ok(general_purpose::STANDARD.encode(&bytes));
            }
        }
    }

    let path_buf = PathBuf::from(path);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_video = matches!(ext.as_str(), "mp4" | "webm" | "mkv" | "avi");

    if is_video {
        let command = app.shell().sidecar("ffmpeg")
            .map_err(|e| format!("Failed to create sidecar command: {}", e))?
            .args([
                "-i", path,
                "-vframes", "1",
                "-vf", "scale=320:-1",
                "-f", "mjpeg",
                "-q:v", "5",
                "pipe:1",
            ]);

        let (mut rx, _child) = command.spawn()
            .map_err(|e| format!("Failed to spawn ffmpeg sidecar: {}", e))?;

        let mut stdout_data = Vec::new();
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => stdout_data.extend_from_slice(&bytes),
                CommandEvent::Error(err) => eprintln!("ffmpeg thumbnail error: {}", err),
                CommandEvent::Terminated(_) => break,
                _ => {}
            }
        }

        if stdout_data.is_empty() {
            return Err("Failed to extract video thumbnail using sidecar".to_string());
        }

        if let Some(ref cp) = cache_path {
            let _ = fs::write(cp, &stdout_data);
        }

        Ok(general_purpose::STANDARD.encode(&stdout_data))
    } else {
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
        let thumb = img.thumbnail(320, 320);
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        thumb.write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

        if let Some(ref cp) = cache_path {
            let _ = fs::write(cp, &buf);
        }

        Ok(general_purpose::STANDARD.encode(&buf))
    }
}

pub fn delete_item(path: &str) -> Result<(), String> {
    if let Some(cache_path) = get_cache_path(path) {
        if cache_path.exists() {
            let _ = fs::remove_file(cache_path);
        }
    }
    fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
}

pub fn open_externally(path: &str) -> Result<(), String> {
    open::that(path).map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}
