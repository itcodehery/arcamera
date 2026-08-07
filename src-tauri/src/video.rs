use std::fs;
use std::path::PathBuf;
use chrono::Local;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub struct VideoRecorder {
    tx: Sender<Vec<u8>>,
    handle: thread::JoinHandle<Result<String, String>>,
}

impl VideoRecorder {
    pub fn start(app: AppHandle, fps: u32) -> Result<Self, String> {
        let videos_dir = dirs::video_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("Arcamera");

        if !videos_dir.exists() {
            fs::create_dir_all(&videos_dir)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filepath = videos_dir.join(format!("video_{}.mp4", timestamp));
        let filepath_str = filepath.to_string_lossy().to_string();

        let command = app.shell().sidecar("ffmpeg")
            .map_err(|e| format!("Failed to create sidecar command: {}", e))?
            .args([
                "-y",
                "-f", "mjpeg",
                "-framerate", &fps.to_string(),
                "-i", "pipe:0",
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-preset", "ultrafast",
                "-crf", "23",
                &filepath_str,
            ]);

        let (mut rx_events, mut child) = command.spawn()
            .map_err(|e| format!("Failed to spawn ffmpeg sidecar: {}", e))?;

        let (tx, rx) = channel::<Vec<u8>>();

        let handle = thread::spawn(move || {
            while let Ok(frame) = rx.recv() {
                if child.write(&frame).is_err() {
                    break;
                }
            }
            
            // Drop child to close stdin
            drop(child);

            // Wait for exit
            let mut success = false;
            while let Some(event) = rx_events.blocking_recv() {
                if let CommandEvent::Terminated(payload) = event {
                    success = payload.code == Some(0);
                    break;
                }
            }

            if !success {
                return Err("ffmpeg encoding failed".to_string());
            }

            Ok(filepath_str)
        });

        Ok(Self { tx, handle })
    }

    pub fn write_frame(&self, frame: Vec<u8>) {
        let _ = self.tx.send(frame);
    }
    
    // Stop will consume self, drop tx, and join thread
    pub fn stop(self) -> Result<String, String> {
        drop(self.tx); // close channel
        match self.handle.join() {
            Ok(res) => res,
            Err(_) => Err("Video recording thread panicked".to_string()),
        }
    }
}
