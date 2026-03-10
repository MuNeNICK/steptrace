export interface SessionStatus {
  state: "Idle" | "Recording" | "Paused" | "Stopped";
  step_count: number;
  buffer_count: number;
  capture_mode: string;
}

export interface StepDto {
  id: string;
  number: number;
  timestamp: string;
  event_type: string;
  caption: string;
  window_title: string | null;
  input_text: string | null;
  has_screenshot: boolean;
}

export interface BufferEntryDto {
  id: string;
  timestamp: string;
  window_title: string | null;
}

export interface WindowInfo {
  id: number;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
}
