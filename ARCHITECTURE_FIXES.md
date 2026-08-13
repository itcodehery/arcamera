# Arcamera Architecture Betterments (Lag Fixes)

This document details the architectural improvements integrated into Arcamera to completely eliminate frame buffering and slow-motion lag.

## The Problem: Infinite Frame Buffering
Initially, Arcamera suffered from a severe "slow-motion" bug where the live preview would gradually fall seconds behind real-time. This occurred due to a lethal combination of two factors:
1. **Backend Queueing:** The backend used a `tokio::sync::broadcast` channel (a FIFO queue) which built up a backlog of frames if the UI lagged.
2. **WebKit Streaming Bug:** The backend fed these frames to the frontend via a `multipart/x-mixed-replace` HTTP stream. WebKitGTK (the engine powering Tauri on Linux) notoriously over-buffers multipart HTTP streams. If it couldn't paint frames at exactly 30fps, it simply queued them up in memory indefinitely, causing massive input lag over time.

## The Solution: Leaky Channels & JS Fetch Loops

We implemented a two-layered defense strategy inspired by the lag-prevention techniques in `cosmic-camera`:

### 1. "Leaky" Watch Channels (Backend)
We migrated the core frame distribution mechanism from a `broadcast` channel to a `tokio::sync::watch` channel.
- **Single-Slot Memory:** A `watch` channel only ever holds exactly **one** value: the absolute most recent frame. 
- **Automatic Frame Dropping:** If the camera captures 5 frames while the frontend is busy, the `watch` channel simply overwrites its single slot 5 times. The old frames are instantly discarded.

### 2. Synchronous JS Fetch Loop (Frontend)
Even with a leaky backend, `multipart/x-mixed-replace` streams are inherently asynchronous and prompt the browser to buffer. To kill the buffering entirely, we dismantled the multipart stream.
- **Single-Frame Endpoint:** We added a lightweight `/frame` endpoint in `axum` that just serves the absolute latest JPEG frame in memory.
- **`requestAnimationFrame` Loop:** In `CameraView.jsx`, we now run a highly optimized JavaScript loop. It `fetch`es the `/frame`, converts it to a Blob, injects it into the `<img>` via `URL.createObjectURL`, and then instantly loops via `requestAnimationFrame`.
- **Zero-Buffering Guarantee:** Because the JS loop *awaits* the HTTP response before requesting the next frame, it is physically impossible for a queue to build up. If the browser lags, it naturally drops frames, perfectly preserving real-time parity.

## Summary of Changes
- **`src-tauri/src/state.rs` & `main.rs`**: Swapped `broadcast` for `watch` channel.
- **`src-tauri/src/server.rs`**: Added `single_frame` endpoint at `/frame`.
- **`src/components/CameraView.jsx`**: Replaced the native stream `src` with an active `requestAnimationFrame` blob-fetching loop.
