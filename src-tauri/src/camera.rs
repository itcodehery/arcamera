use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::AppHandle;
use crate::state::AppState;

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

pub fn start_camera_thread(_app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        use nokhwa::utils::{
            CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
        };
        use nokhwa::Camera;

        let index = CameraIndex::Index(0);
        let mut current_fps = state.target_fps.load(Ordering::Relaxed);
        let target_format =
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, current_fps);
        let requested = RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
            RequestedFormatType::Closest(target_format),
        );

        let mut camera_opt = match Camera::new(index.clone(), requested) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Failed to initialize camera: {}", e);
                return;
            }
        };

        if let Err(e) = camera_opt.as_mut().unwrap().open_stream() {
            eprintln!("Failed to open camera stream: {}", e);
            return;
        }

        let mut is_mjpeg = camera_opt.as_ref().unwrap().camera_format().format() == FrameFormat::MJPEG;
        println!(
            "Camera stream opened. Format: {:?}, using {} path",
            camera_opt.as_ref().unwrap().camera_format(),
            if is_mjpeg { "fast MJPEG" } else { "decode+encode fallback" }
        );

        let mut dht_segment = if is_mjpeg {
            build_standard_dht_segment()
        } else {
            Vec::new()
        };

        loop {
            let requested_fps = state.target_fps.load(Ordering::Relaxed);
            if requested_fps != current_fps {
                println!("Changing camera FPS to {}", requested_fps);
                current_fps = requested_fps;
                
                if let Some(mut cam) = camera_opt.take() {
                    let _ = cam.stop_stream();
                } // cam is dropped here to free the device
                
                let target_format = CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, current_fps);
                let requested = RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
                    RequestedFormatType::Closest(target_format),
                );
                
                match Camera::new(index.clone(), requested) {
                    Ok(mut new_camera) => {
                        if new_camera.open_stream().is_ok() {
                            is_mjpeg = new_camera.camera_format().format() == FrameFormat::MJPEG;
                            dht_segment = if is_mjpeg {
                                build_standard_dht_segment()
                            } else {
                                Vec::new()
                            };
                            camera_opt = Some(new_camera);
                            println!("Camera stream restarted successfully.");
                        } else {
                            eprintln!("Failed to open new camera stream.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to recreate camera: {}", e);
                    }
                }
            }

            if let Some(cam) = camera_opt.as_mut() {
                match cam.frame() {
                    Ok(frame) => {
                        let jpeg_bytes = if is_mjpeg {
                            ensure_jpeg_dht(frame.buffer(), &dht_segment)
                        } else {
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

                        *state.latest_frame.lock().unwrap() = Some(jpeg_bytes.clone());
                        let _ = state.frame_tx.send(jpeg_bytes.clone());

                        if state.is_recording.load(Ordering::Relaxed) {
                            if let Ok(guard) = state.video_recorder.lock() {
                                if let Some(recorder) = guard.as_ref() {
                                    recorder.write_frame(jpeg_bytes);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error capturing frame: {}", e);
                    }
                }
            } else {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
}
