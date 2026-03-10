use serde::Serialize;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::capture::screenshot;
use crate::session::model::*;
use crate::state::AppState;

// --- DTOs for frontend ---

#[derive(Debug, Clone, Serialize)]
pub struct StepDto {
    pub id: String,
    pub number: u32,
    pub timestamp: String,
    pub event_type: String,
    pub caption: String,
    pub window_title: Option<String>,
    pub input_text: Option<String>,
    pub has_screenshot: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BufferEntryDto {
    pub id: String,
    pub timestamp: String,
    pub window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionStatusDto {
    pub state: String,
    pub step_count: usize,
    pub buffer_count: usize,
    pub capture_mode: String,
}

// --- Commands ---

#[tauri::command]
pub fn start_recording(state: State<AppState>, title: String) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    mgr.start(title)
}

#[tauri::command]
pub fn stop_recording(state: State<AppState>) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    mgr.stop()
}

#[tauri::command]
pub fn pause_recording(state: State<AppState>) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    mgr.pause()
}

#[tauri::command]
pub fn resume_recording(state: State<AppState>) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    mgr.resume()
}

#[tauri::command]
pub fn get_session_status(state: State<AppState>) -> Result<SessionStatusDto, String> {
    let mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let (step_count, buffer_count) = match &mgr.session {
        Some(s) => (s.steps.len(), s.buffer.len()),
        None => (0, 0),
    };
    let mode_str = match &mgr.capture_mode {
        CaptureMode::FullScreen => "fullscreen".to_string(),
        CaptureMode::Window(id) => format!("window:{}", id),
        CaptureMode::Region { x, y, w, h } => format!("region:{}x{}+{}+{}", w, h, x, y),
    };
    Ok(SessionStatusDto {
        state: format!("{:?}", mgr.state),
        step_count,
        buffer_count,
        capture_mode: mode_str,
    })
}

#[tauri::command]
pub fn get_steps(state: State<AppState>) -> Result<Vec<StepDto>, String> {
    let mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_ref().ok_or("No active session")?;
    Ok(session
        .steps
        .iter()
        .map(|s| StepDto {
            id: s.id.to_string(),
            number: s.number,
            timestamp: s.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type: format!("{:?}", s.event),
            caption: s.caption.clone(),
            window_title: s.window_title.clone(),
            input_text: s.input_text.clone(),
            has_screenshot: !s.screenshot_raw.is_empty(),
        })
        .collect())
}

#[tauri::command]
pub fn get_step_screenshot(state: State<AppState>, step_id: String) -> Result<String, String> {
    let mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_ref().ok_or("No active session")?;
    let id = Uuid::parse_str(&step_id).map_err(|e| e.to_string())?;
    let step = session
        .steps
        .iter()
        .find(|s| s.id == id)
        .ok_or("Step not found")?;
    let data = step.screenshot_annotated.as_ref().unwrap_or(&step.screenshot_raw);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        data,
    ))
}

#[tauri::command]
pub fn get_buffer_entries(state: State<AppState>) -> Result<Vec<BufferEntryDto>, String> {
    let mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_ref().ok_or("No active session")?;
    Ok(session
        .buffer
        .iter()
        .map(|b| BufferEntryDto {
            id: b.id.to_string(),
            timestamp: b.timestamp.format("%H:%M:%S").to_string(),
            window_title: b.window_title.clone(),
        })
        .collect())
}

#[tauri::command]
pub fn promote_buffer(
    state: State<AppState>,
    buffer_id: String,
    caption: String,
) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let id = Uuid::parse_str(&buffer_id).map_err(|e| e.to_string())?;
    mgr.promote_buffer_entry(id, caption)
}

#[tauri::command]
pub fn update_step_caption(
    state: State<AppState>,
    step_id: String,
    caption: String,
) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_mut().ok_or("No active session")?;
    let id = Uuid::parse_str(&step_id).map_err(|e| e.to_string())?;
    let step = session
        .steps
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or("Step not found")?;
    step.caption = caption;
    Ok(())
}

#[tauri::command]
pub fn delete_step(state: State<AppState>, step_id: String) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_mut().ok_or("No active session")?;
    let id = Uuid::parse_str(&step_id).map_err(|e| e.to_string())?;
    session.steps.retain(|s| s.id != id);
    // Renumber
    for (i, step) in session.steps.iter_mut().enumerate() {
        step.number = i as u32 + 1;
    }
    Ok(())
}

#[tauri::command]
pub fn reorder_steps(state: State<AppState>, step_ids: Vec<String>) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let session = mgr.session.as_mut().ok_or("No active session")?;
    let ids: Vec<Uuid> = step_ids
        .iter()
        .map(|s| Uuid::parse_str(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut reordered = Vec::with_capacity(ids.len());
    for id in &ids {
        let step = session
            .steps
            .iter()
            .find(|s| &s.id == id)
            .ok_or("Step not found")?
            .clone();
        reordered.push(step);
    }
    for (i, step) in reordered.iter_mut().enumerate() {
        step.number = i as u32 + 1;
    }
    session.steps = reordered;
    Ok(())
}

#[tauri::command]
pub fn set_capture_mode(state: State<AppState>, mode: String, params: Option<String>) -> Result<(), String> {
    let mut mgr = state.manager.lock().map_err(|e| e.to_string())?;
    let capture_mode = match mode.as_str() {
        "fullscreen" => CaptureMode::FullScreen,
        "window" => {
            let id: u32 = params
                .ok_or("Window ID required")?
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?;
            CaptureMode::Window(id)
        }
        "region" => {
            let p = params.ok_or("Region params required (x,y,w,h)")?;
            let parts: Vec<i32> = p
                .split(',')
                .map(|s| s.trim().parse().map_err(|e: std::num::ParseIntError| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            if parts.len() != 4 {
                return Err("Region requires 4 params: x,y,w,h".to_string());
            }
            CaptureMode::Region {
                x: parts[0],
                y: parts[1],
                w: parts[2] as u32,
                h: parts[3] as u32,
            }
        }
        _ => return Err(format!("Unknown capture mode: {}", mode)),
    };
    mgr.set_capture_mode(capture_mode);
    Ok(())
}

#[tauri::command]
pub fn get_windows_list() -> Result<Vec<screenshot::WindowInfo>, String> {
    screenshot::list_windows()
}

#[tauri::command]
pub fn switch_to_review(app: AppHandle) -> Result<(), String> {
    // Open review window first
    WebviewWindowBuilder::new(&app, "review", WebviewUrl::App("/review".into()))
        .title("StepTrace - Review")
        .inner_size(1100.0, 700.0)
        .center()
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    // Then close toolbar window
    if let Some(toolbar) = app.get_webview_window("toolbar") {
        let _ = toolbar.close();
    }
    Ok(())
}

#[tauri::command]
pub fn switch_to_toolbar(app: AppHandle) -> Result<(), String> {
    // Open toolbar window first
    WebviewWindowBuilder::new(&app, "toolbar", WebviewUrl::App("/".into()))
        .title("StepTrace")
        .inner_size(460.0, 56.0)
        .center()
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .build()
        .map_err(|e| e.to_string())?;
    // Then close review window
    if let Some(review) = app.get_webview_window("review") {
        let _ = review.close();
    }
    Ok(())
}
