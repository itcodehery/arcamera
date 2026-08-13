use std::sync::Arc;
use axum::{
    extract::State,
    response::Response,
    routing::get,
    Router,
};
use axum::body::{Body, Bytes};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;
use crate::state::AppState;

pub fn start_mjpeg_server(state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let app = Router::new()
            .route("/stream", get(mjpeg_stream))
            .route("/frame", get(single_frame))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:49283").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

async fn single_frame(State(state): State<Arc<AppState>>) -> Response {
    let frame_opt = state.latest_frame.lock().unwrap().clone();
    if let Some(frame) = frame_opt {
        Response::builder()
            .header("Content-Type", "image/jpeg")
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(frame))
            .unwrap()
    } else {
        Response::builder()
            .status(404)
            .body(Body::empty())
            .unwrap()
    }
}

async fn mjpeg_stream(State(state): State<Arc<AppState>>) -> Response {
    let rx = state.frame_tx.subscribe();
    let stream = WatchStream::new(rx).filter_map(|frame| {
        if frame.is_empty() {
            return None;
        }
        let header = format!(
            "--arcamera_frame\r\n\
             Content-Type: image/jpeg\r\n\
             Content-Length: {}\r\n\r\n",
            frame.len()
        );
        let mut data = Vec::with_capacity(header.len() + frame.len() + 2);
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(&frame);
        data.extend_from_slice(b"\r\n");
        Some(Ok::<_, std::io::Error>(Bytes::from(data)))
    });

    let body = Body::from_stream(stream);
    
    Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=arcamera_frame")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Pragma", "no-cache")
        .header("Expires", "0")
        .header("Connection", "close")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}
