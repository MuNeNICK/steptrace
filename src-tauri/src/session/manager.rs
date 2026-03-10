use std::sync::mpsc;

use uuid::Uuid;

use super::model::*;

/// Central orchestrator for recording sessions.
pub struct SessionManager {
    pub state: RecordingState,
    pub session: Option<Session>,
    pub capture_mode: CaptureMode,
    buffer_max_seconds: u64,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
            session: None,
            capture_mode: CaptureMode::default(),
            buffer_max_seconds: 60,
        }
    }

    pub fn start(&mut self, title: String) -> Result<(), String> {
        match self.state {
            RecordingState::Idle | RecordingState::Stopped => {
                self.session = Some(Session::new(title, self.capture_mode.clone()));
                self.state = RecordingState::Recording;
                Ok(())
            }
            _ => Err(format!("Cannot start from state {:?}", self.state)),
        }
    }

    pub fn pause(&mut self) -> Result<(), String> {
        match self.state {
            RecordingState::Recording => {
                self.state = RecordingState::Paused;
                Ok(())
            }
            _ => Err(format!("Cannot pause from state {:?}", self.state)),
        }
    }

    pub fn resume(&mut self) -> Result<(), String> {
        match self.state {
            RecordingState::Paused => {
                self.state = RecordingState::Recording;
                Ok(())
            }
            _ => Err(format!("Cannot resume from state {:?}", self.state)),
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        match self.state {
            RecordingState::Recording | RecordingState::Paused => {
                self.state = RecordingState::Stopped;
                if let Some(ref mut session) = self.session {
                    session.stopped_at = Some(chrono::Local::now());
                }
                Ok(())
            }
            _ => Err(format!("Cannot stop from state {:?}", self.state)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.state == RecordingState::Recording
    }

    pub fn add_step(
        &mut self,
        event: StepEvent,
        screenshot_raw: Vec<u8>,
        screenshot_annotated: Option<Vec<u8>>,
        window_title: Option<String>,
        input_text: Option<String>,
    ) -> Result<(), String> {
        let session = self.session.as_mut().ok_or("No active session")?;
        let number = session.next_step_number();
        let caption = event.auto_description(&window_title);
        let step = Step {
            id: Uuid::new_v4(),
            number,
            timestamp: chrono::Local::now(),
            event,
            caption,
            window_title,
            screenshot_raw,
            screenshot_annotated,
            input_text,
        };
        session.add_step(step);
        Ok(())
    }

    pub fn add_buffer_entry(
        &mut self,
        screenshot: Vec<u8>,
        window_title: Option<String>,
    ) -> Result<(), String> {
        let session = self.session.as_mut().ok_or("No active session")?;
        let entry = BufferEntry::new(screenshot, window_title);

        // Prune old entries beyond buffer_max_seconds
        let cutoff = chrono::Local::now()
            - chrono::Duration::seconds(self.buffer_max_seconds as i64);
        session.buffer.retain(|b| b.timestamp >= cutoff);
        session.buffer.push(entry);
        Ok(())
    }

    pub fn promote_buffer_entry(
        &mut self,
        buffer_id: Uuid,
        caption: String,
    ) -> Result<(), String> {
        let session = self.session.as_mut().ok_or("No active session")?;
        session
            .promote_buffer(buffer_id, caption)
            .map(|_| ())
            .ok_or_else(|| "Buffer entry not found".to_string())
    }

    pub fn set_capture_mode(&mut self, mode: CaptureMode) {
        self.capture_mode = mode.clone();
        if let Some(ref mut session) = self.session {
            session.capture_mode = mode;
        }
    }

    pub fn set_buffer_max_seconds(&mut self, seconds: u64) {
        self.buffer_max_seconds = seconds;
    }
}
