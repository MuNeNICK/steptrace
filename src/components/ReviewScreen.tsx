import { createSignal, createEffect, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  getSteps,
  getStepScreenshot,
  getBufferEntries,
  promoteBuffer,
  updateStepCaption,
  deleteStep,
} from "../lib/ipc";
import type { StepDto, BufferEntryDto } from "../lib/types";

export default function ReviewScreen() {
  const [steps, setSteps] = createSignal<StepDto[]>([]);
  const [bufferEntries, setBufferEntries] = createSignal<BufferEntryDto[]>([]);
  const [selectedStep, setSelectedStep] = createSignal<string | null>(null);
  const [previewSrc, setPreviewSrc] = createSignal<string | null>(null);
  const [showBuffer, setShowBuffer] = createSignal(false);

  const loadData = async () => {
    try {
      const [s, b] = await Promise.all([getSteps(), getBufferEntries()]);
      setSteps(s);
      setBufferEntries(b);
    } catch (e) {
      console.error("Failed to load data:", e);
    }
  };

  createEffect(() => {
    loadData();
  });

  const handleSelect = async (stepId: string) => {
    setSelectedStep(stepId);
    try {
      const b64 = await getStepScreenshot(stepId);
      setPreviewSrc(`data:image/png;base64,${b64}`);
    } catch {
      setPreviewSrc(null);
    }
  };

  const handleCaption = async (stepId: string, caption: string) => {
    await updateStepCaption(stepId, caption);
  };

  const handleDelete = async (stepId: string) => {
    await deleteStep(stepId);
    if (selectedStep() === stepId) {
      setSelectedStep(null);
      setPreviewSrc(null);
    }
    await loadData();
  };

  const handlePromote = async (bufferId: string) => {
    await promoteBuffer(bufferId, "Promoted from buffer");
    await loadData();
  };

  const handleNewRecording = async () => {
    await invoke("switch_to_toolbar");
  };

  return (
    <div class="review">
      <div class="review-header">
        <h2>StepTrace - Review</h2>
        <div class="review-actions">
          <button
            class="btn"
            onClick={() => setShowBuffer(!showBuffer())}
          >
            Buffer ({bufferEntries().length})
          </button>
          <button class="btn btn-accent">Export</button>
          <button class="btn" onClick={handleNewRecording}>
            New Recording
          </button>
        </div>
      </div>

      <div class="review-body">
        <div class="step-panel">
          <div class="step-panel-header">
            Steps ({steps().length})
          </div>
          <For each={steps()}>
            {(step) => (
              <div
                class={`step-item ${selectedStep() === step.id ? "selected" : ""}`}
                onClick={() => handleSelect(step.id)}
              >
                <div class="step-item-header">
                  <span class="step-num">#{step.number}</span>
                  <span class="step-time">{step.timestamp}</span>
                  <button
                    class="step-delete"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(step.id);
                    }}
                  >
                    ✕
                  </button>
                </div>
                <input
                  class="step-caption"
                  value={step.caption}
                  onChange={(e) => handleCaption(step.id, e.currentTarget.value)}
                  onClick={(e) => e.stopPropagation()}
                />
                <Show when={step.input_text}>
                  <code class="step-cmd">{step.input_text}</code>
                </Show>
              </div>
            )}
          </For>
          <Show when={steps().length === 0}>
            <div class="empty-steps">No steps recorded</div>
          </Show>
        </div>

        <div class="preview-panel">
          <Show
            when={previewSrc()}
            fallback={<span class="preview-empty">Select a step to preview screenshot</span>}
          >
            <img src={previewSrc()!} alt="Screenshot" class="preview-img" />
          </Show>
        </div>
      </div>

      <Show when={showBuffer()}>
        <div class="buffer-panel">
          <div class="buffer-header">
            <span>Buffer Timeline</span>
          </div>
          <div class="buffer-grid">
            <For each={bufferEntries()}>
              {(entry) => (
                <div class="buffer-item">
                  <span class="buffer-time">{entry.timestamp}</span>
                  <span>{entry.window_title ?? "—"}</span>
                  <button class="buffer-promote" onClick={() => handlePromote(entry.id)}>
                    + Step
                  </button>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}
