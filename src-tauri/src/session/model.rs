use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub title: String,
    pub started_at: DateTime<Local>,
    pub stopped_at: Option<DateTime<Local>>,
    pub capture_mode: CaptureMode,
    pub steps: Vec<Step>,
    pub buffer: Vec<BufferEntry>,
}

impl Session {
    pub fn new(title: String, capture_mode: CaptureMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            started_at: Local::now(),
            stopped_at: None,
            capture_mode,
            steps: Vec::new(),
            buffer: Vec::new(),
        }
    }

    pub fn next_step_number(&self) -> u32 {
        self.steps.len() as u32 + 1
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    pub fn promote_buffer(&mut self, buffer_id: Uuid, caption: String) -> Option<&Step> {
        let entry = self.buffer.iter().find(|b| b.id == buffer_id)?;
        let step = Step {
            id: Uuid::new_v4(),
            number: self.next_step_number(),
            timestamp: entry.timestamp,
            event: StepEvent::Manual,
            caption,
            window_title: entry.window_title.clone(),
            screenshot_raw: entry.screenshot.clone(),
            screenshot_annotated: None,
            input_text: None,
        };
        self.steps.push(step);
        self.steps.last()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub number: u32,
    pub timestamp: DateTime<Local>,
    pub event: StepEvent,
    pub caption: String,
    pub window_title: Option<String>,
    /// Raw screenshot bytes (PNG)
    #[serde(skip)]
    pub screenshot_raw: Vec<u8>,
    /// Annotated screenshot bytes (PNG) with markers
    #[serde(skip)]
    pub screenshot_annotated: Option<Vec<u8>>,
    /// Captured text input (e.g., command typed before Enter)
    pub input_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Local>,
    pub window_title: Option<String>,
    #[serde(skip)]
    pub screenshot: Vec<u8>,
}

impl BufferEntry {
    pub fn new(screenshot: Vec<u8>, window_title: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Local::now(),
            window_title,
            screenshot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepEvent {
    LeftClick { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    KeyEnter,
    WindowSwitch,
    Manual,
}

impl StepEvent {
    pub fn auto_description(&self, window_title: &Option<String>) -> String {
        let target = window_title
            .as_deref()
            .unwrap_or("unknown");
        match self {
            StepEvent::LeftClick { x, y } => {
                format!("Left-clicked on '{}' at ({}, {})", target, x, y)
            }
            StepEvent::RightClick { x, y } => {
                format!("Right-clicked on '{}' at ({}, {})", target, x, y)
            }
            StepEvent::KeyEnter => {
                format!("Pressed Enter in '{}'", target)
            }
            StepEvent::WindowSwitch => {
                format!("Switched to '{}'", target)
            }
            StepEvent::Manual => "Manual capture".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMode {
    FullScreen,
    Window(u32),
    Region { x: i32, y: i32, w: u32, h: u32 },
}

impl Default for CaptureMode {
    fn default() -> Self {
        CaptureMode::FullScreen
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Stopped,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState::Idle
    }
}
