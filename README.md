# Arcamera

Arcamera is a fast, modernized, cross-platform camera application built with Tauri v2, Rust, and React+Vite. It features low-latency MJPEG video streaming, high-quality photo capture, video recording via bundled FFmpeg, and a gorgeous Material 3 Expressive UI.

## Features

- **Live Camera Feed**: Instant, local video streaming via an embedded Axum HTTP server using fast MJPEG.
- **Photo Capture**: Capture and save high-resolution photos seamlessly.
- **Video Recording**: High-performance video recording using an FFmpeg sidecar, eliminating in-memory buffering bottlenecks.
- **Modern UI**: A sleek, component-driven React interface featuring Dark Mode, rule-of-thirds grid overlays, interactive toasts, and smooth micro-animations.
- **Built-in Gallery**: View and manage your captured photos and videos from within the app.
- **Configurable Settings**: Toggle camera mirroring, grid overlays, and view storage locations.

## Tech Stack

- **Frontend**: React 18, Vite, standard CSS
- **Backend**: Rust, Tauri v2 (tauri-plugin-shell)
- **Local Streaming**: Tokio broadcast channels, Axum
- **Video Processing**: FFmpeg (bundled as a sidecar binary)

## Installation and Setup

### Prerequisites

1. **Rust**: Ensure you have [Rust installed](https://rustup.rs/).
2. **Node.js**: Required for the Vite frontend toolchain.
3. **FFmpeg**: You must place an `ffmpeg` binary inside `src-tauri/bin/` named exactly as your target triple (e.g., `ffmpeg-x86_64-unknown-linux-gnu`) for the video recording feature to function locally. (Note: The CI/CD pipeline downloads this automatically for release builds).

### Running Locally

```bash
# 1. Install frontend dependencies
npm install

# 2. Run the application in development mode (hot-reloading enabled)
npm run tauri dev
```

### Building for Production

```bash
# Compile and bundle the application
npm run tauri build
```
The compiled binaries (AppImage, deb, etc.) will be available in `src-tauri/target/release/bundle/`.

## Architecture

- **`src/`**: Contains the Vite + React frontend codebase.
- **`src-tauri/`**: Contains the Rust backend, including the Axum local server (`server.rs`, `camera.rs`), FFmpeg recording logic (`video.rs`), and Tauri IPC handlers (`commands.rs`).

## CI/CD and Releases

Arcamera comes pre-configured with a GitHub Actions workflow. When you push a new version tag (e.g., `v1.0.0`), the pipeline will automatically build the application and upload the AppImage and DEB bundles to a GitHub Release draft.
