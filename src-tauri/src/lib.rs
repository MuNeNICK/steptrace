mod annotate;
mod capture;
mod commands;
mod input;
mod session;
mod state;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Start the recording engine on a background thread
            let handle = app.handle().clone();
            start_recording_engine(handle);

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::get_session_status,
            commands::get_steps,
            commands::get_step_screenshot,
            commands::get_buffer_entries,
            commands::promote_buffer,
            commands::update_step_caption,
            commands::delete_step,
            commands::reorder_steps,
            commands::set_capture_mode,
            commands::get_windows_list,
            commands::switch_to_review,
            commands::switch_to_toolbar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_recording_engine(handle: tauri::AppHandle) {
    // Input listener thread
    let input_rx = input::listener::start_listener();

    // Main recording loop: processes input events and triggers captures
    thread::spawn(move || {
        let mut keystroke_buf = String::new();
        let mut last_buffer_capture = std::time::Instant::now();
        let buffer_interval = Duration::from_secs(3);

        loop {
            // Check for input events (non-blocking with timeout)
            match input_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    let state = handle.state::<AppState>();
                    let mgr = state.manager.lock().unwrap();
                    if !mgr.is_recording() {
                        drop(mgr);
                        continue;
                    }
                    let capture_mode = mgr.capture_mode.clone();
                    drop(mgr);

                    match event {
                        input::listener::InputEvent::LeftClick { x, y } => {
                            handle_click(&handle, &capture_mode, x, y, false);
                        }
                        input::listener::InputEvent::RightClick { x, y } => {
                            handle_click(&handle, &capture_mode, x, y, true);
                        }
                        input::listener::InputEvent::KeyEnter => {
                            let input_text = if keystroke_buf.is_empty() {
                                None
                            } else {
                                Some(std::mem::take(&mut keystroke_buf))
                            };
                            handle_enter(&handle, &capture_mode, input_text);
                        }
                        input::listener::InputEvent::KeyPress(ch) => {
                            keystroke_buf.push(ch);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic buffer capture (every 3 seconds, not every 100ms)
                    if last_buffer_capture.elapsed() >= buffer_interval {
                        let state = handle.state::<AppState>();
                        let mgr = state.manager.lock().unwrap();
                        if mgr.is_recording() {
                            let mode = mgr.capture_mode.clone();
                            drop(mgr);
                            if let Ok(screenshot) = capture::screenshot::capture(&mode) {
                                let state = handle.state::<AppState>();
                                let mut mgr = state.manager.lock().unwrap();
                                let _ = mgr.add_buffer_entry(screenshot, None);
                            }
                        }
                        last_buffer_capture = std::time::Instant::now();
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    log::error!("Input listener disconnected");
                    break;
                }
            }
        }
    });
}

fn handle_click(handle: &tauri::AppHandle, mode: &session::model::CaptureMode, x: f64, y: f64, right: bool) {
    // Small delay for UI to settle after click
    thread::sleep(Duration::from_millis(50));

    if let Ok(screenshot) = capture::screenshot::capture(mode) {
        // TODO: annotate the image (click marker + window highlight)

        let event = if right {
            session::model::StepEvent::RightClick { x: x as i32, y: y as i32 }
        } else {
            session::model::StepEvent::LeftClick { x: x as i32, y: y as i32 }
        };

        let state = handle.state::<AppState>();
        let mut mgr = state.manager.lock().unwrap();
        let _ = mgr.add_step(event, screenshot, None, None, None);
    }
}

fn handle_enter(handle: &tauri::AppHandle, mode: &session::model::CaptureMode, input_text: Option<String>) {
    thread::sleep(Duration::from_millis(100));

    if let Ok(screenshot) = capture::screenshot::capture(mode) {
        let state = handle.state::<AppState>();
        let mut mgr = state.manager.lock().unwrap();
        let _ = mgr.add_step(
            session::model::StepEvent::KeyEnter,
            screenshot,
            None,
            None,
            input_text,
        );
    }
}
