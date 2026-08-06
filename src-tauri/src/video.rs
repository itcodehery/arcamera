use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use chrono::Local;

/// Encode collected JPEG frames into an MP4 video using ffmpeg.
pub fn save_video_ffmpeg(frames: &[Vec<u8>], fps: u32) -> Result<String, String> {
    // Phase 1 cross-platform fix using dirs crate
    let videos_dir = dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Arcamera");

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
