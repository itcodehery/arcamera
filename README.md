# Arcamera

A lightweight, native camera application built with Rust and Tauri.

## Features
- **Photo Capture**: Take photos using your computer's webcam.
- **Video Recording**: Record high-quality videos natively via FFmpeg.
- **Gallery**: View your captured photos and videos seamlessly in an integrated grid view.
- **Lightbox**: Full-screen preview of images with options to open externally or delete them.
- **Settings**:
  - Mirror preview (useful for front-facing webcams).
  - Rule of thirds grid overlay.
- **Fast Performance**: Raw MJPEG extraction directly from V4L2 cameras avoiding expensive full re-encodes.

## Requirements
- Rust & Cargo
- Tauri prerequisites (WebKitGTK, build-essential)
- FFmpeg (required for video recording)

## Development
Run the app locally with:
```bash
cargo tauri dev
```

## Storage Locations
- Photos: `~/Pictures/Arcamera`
- Videos: `~/Videos/Arcamera`
