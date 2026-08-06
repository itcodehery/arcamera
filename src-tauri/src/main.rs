// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod camera;
mod video;
mod gallery;
mod commands;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use state::AppState;

fn main() {
    let app_state = Arc::new(AppState {
        is_recording: AtomicBool::new(false),
        recording_start: Mutex::new(None),
        video_recorder: Mutex::new(None),
    });

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
