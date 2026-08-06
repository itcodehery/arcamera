use std::fs;
use std::path::PathBuf;
use chrono::Local;
use std::process::{Command, Stdio};
use base64::{Engine as _, engine::general_purpose};

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

    // Phase 1 cross-platform fix using dirs crate
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

pub fn get_thumbnail(path: &str) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_video = matches!(ext.as_str(), "mp4" | "webm" | "mkv" | "avi");

    if is_video {
        // Extract first frame with ffmpeg
        let output = Command::new("ffmpeg")
            .args([
                "-i", path, "-vframes", "1", "-vf", "scale=400:-1", "-f", "mjpeg",
                "-q:v", "5", "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| format!("Failed to extract video thumbnail: {}", e))?;

        if output.stdout.is_empty() {
            return Err("Failed to extract video frame".to_string());
        }
        Ok(general_purpose::STANDARD.encode(&output.stdout))
    } else {
        let bytes =
            fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
        Ok(general_purpose::STANDARD.encode(&bytes))
    }
}

pub fn delete_item(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
}

pub fn open_externally(path: &str) -> Result<(), String> {
    // Phase 1 cross-platform fix using open crate
    open::that(path).map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}
