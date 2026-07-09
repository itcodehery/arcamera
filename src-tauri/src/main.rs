// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{Engine as _, engine::general_purpose};
use chrono::Local;
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};

/// Shared state between the camera thread and Tauri commands.
struct AppState {
    is_recording: AtomicBool,
    recorded_frames: Mutex<Vec<Vec<u8>>>,
    recording_start: Mutex<Option<Instant>>,
}

#[derive(serde::Serialize)]
struct GalleryItem {
    path: String,
    filename: String,
    is_video: bool,
    size_bytes: u64,
    modified: String,
}

// ---------------------------------------------------------------------------
// JPEG DHT (Huffman Table) injection
// ---------------------------------------------------------------------------
// V4L2 MJPEG cameras often emit JPEG frames without the DHT segment.
// Browsers require DHT to decode JPEG, so we inject the standard baseline
// Huffman tables (from JPEG spec ITU-T.81 Annex K) when they're missing.
// This is a cheap byte-copy operation (~0.1ms) vs the old decode+re-encode
// approach (~50-100ms in debug mode).

/// Build the standard JPEG DHT segment containing all 4 baseline Huffman tables.
/// Returns a 420-byte segment: FF C4 (marker) + 01 A2 (length) + table data.
fn build_standard_dht_segment() -> Vec<u8> {
    let mut seg = Vec::with_capacity(420);
    // DHT marker
    seg.extend_from_slice(&[0xFF, 0xC4]);
    // Length placeholder (filled at the end)
    let len_pos = seg.len();
    seg.extend_from_slice(&[0x00, 0x00]);

    // Table 0: DC Luminance (class=0, id=0)
    seg.push(0x00);
    seg.extend_from_slice(&[0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]);
    seg.extend_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    // Table 1: AC Luminance (class=1, id=0)
    seg.push(0x10);
    seg.extend_from_slice(&[0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7D]);
    seg.extend_from_slice(&[
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
        0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
        0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
        0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
        0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
        0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
        0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
        0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
        0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
        0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
        0xF9, 0xFA,
    ]);

    // Table 2: DC Chrominance (class=0, id=1)
    seg.push(0x01);
    seg.extend_from_slice(&[0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
    seg.extend_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    // Table 3: AC Chrominance (class=1, id=1)
    seg.push(0x11);
    seg.extend_from_slice(&[0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77]);
    seg.extend_from_slice(&[
        0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
        0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0,
        0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
        0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
        0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
        0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
        0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3,
        0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
        0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6,
        0xF7, 0xF8, 0xF9, 0xFA,
    ]);

    // Fill in the length field (everything after the 0xFF 0xC4 marker)
    let total_len = (seg.len() - 2) as u16;
    seg[len_pos] = (total_len >> 8) as u8;
    seg[len_pos + 1] = (total_len & 0xFF) as u8;

    seg
}

/// Walk JPEG markers to check for DHT presence and find SOS position.
/// Returns (has_dht, sos_byte_offset).
fn scan_jpeg_markers(jpeg: &[u8]) -> (bool, Option<usize>) {
    let mut has_dht = false;
    let mut i = 0;
    while i < jpeg.len().saturating_sub(1) {
        if jpeg[i] == 0xFF {
            let marker = jpeg[i + 1];
            if marker == 0xC4 {
                has_dht = true;
            }
            if marker == 0xDA {
                return (has_dht, Some(i));
            }
            // SOI, EOI, RST0-7 and stuffed bytes have no length field
            if marker == 0xD8 || marker == 0xD9
                || (marker >= 0xD0 && marker <= 0xD7)
                || marker == 0x00
            {
                i += 2;
            } else if i + 3 < jpeg.len() {
                let len = ((jpeg[i + 2] as usize) << 8) | (jpeg[i + 3] as usize);
                i += 2 + len;
            } else {
                break;
            }
        } else {
            i += 1;
        }
    }
    (has_dht, None)
}

/// Ensure the JPEG frame has DHT tables. If not, inject standard ones before SOS.
fn ensure_jpeg_dht(raw: &[u8], dht_segment: &[u8]) -> Vec<u8> {
    let (has_dht, sos_pos) = scan_jpeg_markers(raw);
    if has_dht {
        return raw.to_vec();
    }
    if let Some(sos) = sos_pos {
        let mut out = Vec::with_capacity(raw.len() + dht_segment.len());
        out.extend_from_slice(&raw[..sos]);
        out.extend_from_slice(dht_segment);
        out.extend_from_slice(&raw[sos..]);
        out
    } else {
        // Malformed JPEG — pass through as-is
        raw.to_vec()
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn save_photo(data_url: String) -> Result<String, String> {
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

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let pictures_dir = PathBuf::from(home).join("Pictures").join("Arcamera");

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
fn start_recording(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.recorded_frames.lock().unwrap().clear();
    *state.recording_start.lock().unwrap() = Some(Instant::now());
    state.is_recording.store(true, Ordering::Release);
    Ok(())
}

#[tauri::command]
fn stop_recording(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    state.is_recording.store(false, Ordering::Release);

    let frames: Vec<Vec<u8>> = std::mem::take(&mut *state.recorded_frames.lock().unwrap());
    let start = state.recording_start.lock().unwrap().take();

    if frames.is_empty() {
        return Err("No frames were recorded".to_string());
    }

    // Calculate actual fps from elapsed time
    let duration = start
        .map(|s| s.elapsed())
        .unwrap_or(std::time::Duration::from_secs(1));
    let fps = (frames.len() as f64 / duration.as_secs_f64()).round() as u32;
    let fps = fps.max(1).min(60);

    save_video_ffmpeg(&frames, fps)
}

// ---------------------------------------------------------------------------
// Gallery commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn list_gallery_items() -> Result<Vec<GalleryItem>, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut items = Vec::new();

    let dirs = [
        PathBuf::from(&home).join("Pictures").join("Arcamera"),
        PathBuf::from(&home).join("Videos").join("Arcamera"),
    ];

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

#[tauri::command]
fn get_thumbnail(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
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
                "-i", &path, "-vframes", "1", "-vf", "scale=400:-1", "-f", "mjpeg",
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

#[tauri::command]
fn delete_gallery_item(path: String) -> Result<(), String> {
    fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))
}

#[tauri::command]
fn open_file_externally(path: String) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(())
}

/// Encode collected JPEG frames into an MP4 video using ffmpeg.
fn save_video_ffmpeg(frames: &[Vec<u8>], fps: u32) -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let videos_dir = PathBuf::from(home).join("Videos").join("Arcamera");

    if !videos_dir.exists() {
        fs::create_dir_all(&videos_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filepath = videos_dir.join(format!("video_{}.mp4", timestamp));

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "mjpeg",
            "-framerate",
            &fps.to_string(),
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "ultrafast",
            "-crf",
            "23",
            filepath.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg (is it installed?): {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open ffmpeg stdin")?;
        for frame in frames {
            stdin
                .write_all(frame)
                .map_err(|e| format!("Failed to write frame to ffmpeg: {}", e))?;
        }
    }
    // stdin is dropped here, closing the pipe so ffmpeg knows input is done

    let output = child
        .wait_with_output()
        .map_err(|e| format!("ffmpeg process error: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg encoding failed: {}", stderr));
    }

    Ok(filepath.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// Camera thread
// ---------------------------------------------------------------------------

fn start_camera_thread(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        use nokhwa::utils::{
            CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
        };
        use nokhwa::Camera;

        let index = CameraIndex::Index(0);
        let target_format =
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, 30);
        let requested = RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
            RequestedFormatType::Closest(target_format),
        );

        let mut camera = match Camera::new(index, requested) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to initialize camera: {}", e);
                return;
            }
        };

        if let Err(e) = camera.open_stream() {
            eprintln!("Failed to open camera stream: {}", e);
            return;
        }

        let format = camera.camera_format();
        let is_mjpeg = format.format() == FrameFormat::MJPEG;
        println!(
            "Camera stream opened. Format: {:?}, using {} path",
            format,
            if is_mjpeg { "fast MJPEG" } else { "decode+encode fallback" }
        );

        // Build the DHT segment once (only needed for MJPEG fast path)
        let dht_segment = if is_mjpeg {
            build_standard_dht_segment()
        } else {
            Vec::new()
        };

        loop {
            match camera.frame() {
                Ok(frame) => {
                    let jpeg_bytes = if is_mjpeg {
                        // Fast path: use raw MJPEG bytes, inject DHT if missing
                        ensure_jpeg_dht(frame.buffer(), &dht_segment)
                    } else {
                        // Fallback: decode to RGB then re-encode as JPEG
                        match frame.decode_image::<nokhwa::pixel_format::RgbFormat>() {
                            Ok(decoded) => {
                                let width = decoded.width();
                                let height = decoded.height();
                                let mut buf = Vec::with_capacity(300_000);
                                let mut cursor = Cursor::new(&mut buf);
                                let mut encoder =
                                    image::codecs::jpeg::JpegEncoder::new_with_quality(
                                        &mut cursor,
                                        75,
                                    );
                                if encoder
                                    .encode(
                                        decoded.as_raw(),
                                        width,
                                        height,
                                        image::ExtendedColorType::Rgb8,
                                    )
                                    .is_ok()
                                {
                                    buf
                                } else {
                                    continue;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to decode frame: {}", e);
                                continue;
                            }
                        }
                    };

                    // Buffer frames while recording
                    if state.is_recording.load(Ordering::Relaxed) {
                        if let Ok(mut frames) = state.recorded_frames.lock() {
                            frames.push(jpeg_bytes.clone());
                        }
                    }

                    let b64 = general_purpose::STANDARD.encode(&jpeg_bytes);
                    let _ = app.emit("camera-frame", b64);
                }
                Err(e) => {
                    eprintln!("Error capturing frame: {}", e);
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let app_state = Arc::new(AppState {
        is_recording: AtomicBool::new(false),
        recorded_frames: Mutex::new(Vec::new()),
        recording_start: Mutex::new(None),
    });

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            start_camera_thread(handle, app_state.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_photo,
            start_recording,
            stop_recording,
            list_gallery_items,
            get_thumbnail,
            delete_gallery_item,
            open_file_externally,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
