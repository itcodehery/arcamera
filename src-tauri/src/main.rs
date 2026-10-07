// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod camera;
mod video;
mod gallery;
mod commands;
mod server;

use std::sync::atomic::{AtomicBool, AtomicU16};
use std::sync::{Arc, Mutex};
use state::AppState;

fn main() {
    let (frame_tx, _) = tokio::sync::watch::channel(Vec::new());
    let app_state = Arc::new(AppState {
        is_recording: AtomicBool::new(false),
        recording_start: Mutex::new(None),
        video_recorder: Mutex::new(None),
        frame_tx,
        latest_frame: Mutex::new(None),
        server_port: AtomicU16::new(49283),
        target_fps: std::sync::atomic::AtomicU32::new(30),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            server::start_mjpeg_server(app_state.clone());
            camera::start_camera_thread(handle, app_state.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_photo,
            commands::start_recording,
            commands::stop_recording,
            commands::list_gallery_items,
            commands::get_thumbnail,
            commands::delete_gallery_item,
            commands::open_file_externally,
            commands::get_stream_port,
            commands::set_camera_fps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
