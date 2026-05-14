import { listen } from "@tauri-apps/api/event";
import { ref } from "vue";
import { api } from "../helpers/api";
import type { ApplyProgressEvent } from "../types";
import { useAppState } from "./useAppState";
import { useTaskStatus } from "./useTaskStatus";

const applyProgress = ref<ApplyProgressEvent | null>(null);

export function useReviewPlan() {
  const state = useAppState();
  const task = useTaskStatus();

  async function createPreview() {
    if (!state.selectedUserId.value) return false;

    const plan = await task.runTask("Creating preview", () =>
      api.createPreviewPlan(state.selectedUserId.value, state.effectiveCandidates.value, state.options.value)
    );
    if (!plan) return false;

    state.previewPlan.value = plan;
    state.applyResult.value = null;
    state.step.value = "review";
    return true;
  }

  async function applyPreview() {
    if (!state.previewPlan.value) return;

    applyProgress.value = null;
    const unlisten = await listen<ApplyProgressEvent>("apply-progress", (event) => {
      applyProgress.value = event.payload;
    });

    const result = await task.runTask("Applying changes", () =>
      api.applyPlan(state.previewPlan.value!, state.effectiveCandidates.value, state.options.value)
    );

    unlisten();
    applyProgress.value = null;

    if (result) {
      state.applyResult.value = result;
    }
  }

  return {
    createPreview,
    applyPreview,
    applyProgress
  };
}
