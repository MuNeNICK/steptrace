use std::sync::Mutex;

use crate::session::manager::SessionManager;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub manager: Mutex<SessionManager>,
    /// Keystroke buffer: accumulated chars between Enter presses
    pub keystroke_buffer: Mutex<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            manager: Mutex::new(SessionManager::new()),
            keystroke_buffer: Mutex::new(String::new()),
        }
    }
}
