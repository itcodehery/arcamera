use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use chrono::Local;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct VideoRecorder {
    tx: Sender<Vec<u8>>,
    handle: thread::JoinHandle<Result<String, String>>,
}

impl VideoRecorder {
    pub fn start(fps: u32) -> Result<Self, String> {
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

        let mut child = Command::new("ffmpeg")
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
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ffmpeg (is it installed?): {}", e))?;

        let mut stdin = child.stdin.take().ok_or("Failed to open ffmpeg stdin")?;
        
        let (tx, rx) = channel::<Vec<u8>>();

        let handle = thread::spawn(move || {
            // Write frames as they come in
            while let Ok(frame) = rx.recv() {
                if stdin.write_all(&frame).is_err() {
                    break;
                }
            }
            // stdin is dropped here, closing the pipe so ffmpeg knows input is done
            drop(stdin);

            let output = child
                .wait_with_output()
                .map_err(|e| format!("ffmpeg process error: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("ffmpeg encoding failed: {}", stderr));
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
