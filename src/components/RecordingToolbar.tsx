import { createSignal, createEffect, onCleanup, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  startRecording,
  stopRecording,
  pauseRecording,
  resumeRecording,
  getSessionStatus,
} from "../lib/ipc";
import type { SessionStatus } from "../lib/types";

export default function RecordingToolbar() {
  const [status, setStatus] = createSignal<SessionStatus>({
    state: "Idle",
    step_count: 0,
    buffer_count: 0,
    capture_mode: "fullscreen",
  });
  const [elapsed, setElapsed] = createSignal(0);
  let elapsedTimer: number | undefined;

  // Poll session status
  createEffect(() => {
    const timer = window.setInterval(async () => {
      try {
        setStatus(await getSessionStatus());
      } catch {}
    }, 500);
    onCleanup(() => clearInterval(timer));
  });

  const state = () => status().state;

  const handleStart = async () => {
    await startRecording("Recording");
    setElapsed(0);
    elapsedTimer = window.setInterval(() => setElapsed((e) => e + 1), 1000);
  };

  const handleStop = async () => {
    await stopRecording();
    if (elapsedTimer) clearInterval(elapsedTimer);
    await invoke("switch_to_review");
  };

  const handlePause = async () => {
    await pauseRecording();
    if (elapsedTimer) clearInterval(elapsedTimer);
  };

  const handleResume = async () => {
    await resumeRecording();
    elapsedTimer = window.setInterval(() => setElapsed((e) => e + 1), 1000);
  };

  const fmt = (s: number) => {
    const m = Math.floor(s / 60).toString().padStart(2, "0");
    const sec = (s % 60).toString().padStart(2, "0");
    return `${m}:${sec}`;
  };

  // Allow dragging the toolbar window
  const onDrag = (e: MouseEvent) => {
    if (e.buttons === 1) {
      import("@tauri-apps/api/webviewWindow").then(({ getCurrentWebviewWindow }) => {
        getCurrentWebviewWindow().startDragging();
      });
    }
  };

  return (
    <div class="toolbar" onMouseDown={onDrag}>
      <div class="toolbar-brand">ST</div>

      <Show when={state() === "Idle"}>
        <button class="tb-btn rec" onClick={handleStart}>
          <span class="rec-dot" /> Record
        </button>
      </Show>

      <Show when={state() === "Recording" || state() === "Paused"}>
        <Show when={state() === "Recording"}>
          <span class="rec-indicator">● REC</span>
          <button class="tb-btn" onClick={handlePause}>Pause</button>
        </Show>
        <Show when={state() === "Paused"}>
          <span class="pause-indicator">❚❚</span>
          <button class="tb-btn" onClick={handleResume}>Resume</button>
        </Show>
        <button class="tb-btn stop" onClick={handleStop}>Stop</button>
        <span class="timer">{fmt(elapsed())}</span>
        <span class="steps">Steps: {status().step_count}</span>
      </Show>
    </div>
  );
}
