import { useAppState } from "./useAppState";
import { useTaskStatus } from "./useTaskStatus";
import { api } from "../helpers/api";

export function useReviewPlan() {
  const state = useAppState();
  const task = useTaskStatus();

  async function createPreview() {
    if (!state.selectedUserId.value) return false;

    const plan = await task.runTask("Creating preview", () =>
      api.createPreviewPlan(state.selectedUserId.value, state.selectedCandidates.value, state.options.value)
    );
    if (!plan) return false;

    state.previewPlan.value = plan;
    state.applyResult.value = null;
    state.step.value = "review";
    return true;
  }

  async function applyPreview() {
    if (!state.previewPlan.value) return;

    const result = await task.runTask("Applying changes", () =>
      api.applyPlan(state.previewPlan.value!, state.selectedCandidates.value, state.options.value)
    );
    if (result) {
      state.applyResult.value = result;
    }
  }

  return {
    createPreview,
    applyPreview
  };
}
