import { invoke } from "@tauri-apps/api/core";
import type {
  SessionStatus,
  StepDto,
  BufferEntryDto,
  WindowInfo,
} from "./types";

export const startRecording = (title: string) =>
  invoke<void>("start_recording", { title });

export const stopRecording = () => invoke<void>("stop_recording");

export const pauseRecording = () => invoke<void>("pause_recording");

export const resumeRecording = () => invoke<void>("resume_recording");

export const getSessionStatus = () =>
  invoke<SessionStatus>("get_session_status");

export const getSteps = () => invoke<StepDto[]>("get_steps");

export const getStepScreenshot = (stepId: string) =>
  invoke<string>("get_step_screenshot", { stepId });

export const getBufferEntries = () =>
  invoke<BufferEntryDto[]>("get_buffer_entries");

export const promoteBuffer = (bufferId: string, caption: string) =>
  invoke<void>("promote_buffer", { bufferId, caption });

export const updateStepCaption = (stepId: string, caption: string) =>
  invoke<void>("update_step_caption", { stepId, caption });

export const deleteStep = (stepId: string) =>
  invoke<void>("delete_step", { stepId });

export const reorderSteps = (stepIds: string[]) =>
  invoke<void>("reorder_steps", { stepIds });

export const setCaptureMode = (mode: string, params?: string) =>
  invoke<void>("set_capture_mode", { mode, params });

export const getWindowsList = () =>
  invoke<WindowInfo[]>("get_windows_list");
