<script setup lang="ts">
import { ArrowRight, Check } from "@lucide/vue";
import { computed } from "vue";
import BottomActionBar from "../../../components/BottomActionBar.vue";
import { useAppState } from "../../../composables/useAppState";
import { useReviewPlan } from "../../../composables/useReviewPlan";
import { useTaskStatus } from "../../../composables/useTaskStatus";
import ArtworkView from "./ArtworkView.vue";
import ReviewView from "./ReviewView.vue";
import SourcesView from "./SourcesView.vue";

const state = useAppState();
const reviewPlan = useReviewPlan();
const task = useTaskStatus();

const nextLabel = computed(() => {
  if (state.step.value === "review") return state.applyResult.value ? "Applied" : "Apply";
  return "Continue";
});

const nextDisabled = computed(() => {
  if (task.loading.value) return true;
  if (state.step.value === "sources") return state.selectedCandidates.value.length === 0;
  if (state.step.value === "review") return !state.previewPlan.value || !!state.applyResult.value;
  return false;
});

function goBack() {
  if (state.step.value === "review") {
    state.step.value = "artwork";
  } else if (state.step.value === "artwork") {
    state.step.value = "sources";
  }
}

async function goNext() {
  if (state.step.value === "sources") {
    state.step.value = "artwork";
    return;
  }

  if (state.step.value === "artwork") {
    await reviewPlan.createPreview();
    return;
  }

  await reviewPlan.applyPreview();
}
</script>

<template>
  <div class="grid content-start gap-3">
    <SourcesView v-if="state.step.value === 'sources'" />

    <ArtworkView v-else-if="state.step.value === 'artwork'" />

    <ReviewView
      v-else
      :plan="state.previewPlan.value"
      :apply-result="state.applyResult.value"
    />

    <BottomActionBar
      :show-back="state.step.value !== 'sources'"
      :next-label="nextLabel"
      :next-disabled="nextDisabled"
      @back="goBack"
      @next="goNext"
    >
      <template #next-icon>
        <Check v-if="state.step.value === 'review'" :size="18" />
        <ArrowRight v-else :size="16" />
      </template>
    </BottomActionBar>
  </div>
</template>
