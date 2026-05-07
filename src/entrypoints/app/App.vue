<script setup lang="ts">
import { computed } from "vue";
import { useRouter, useRoute, RouterView } from "vue-router";
import AppFrame from "../../components/AppFrame.vue";
import { useAppState } from "../../composables/useAppState";
import { useReviewPlan } from "../../composables/useReviewPlan";
import { useTaskStatus } from "../../composables/useTaskStatus";

const router = useRouter();
const route = useRoute();
const state = useAppState();
const reviewPlan = useReviewPlan();
const task = useTaskStatus();
const settingsOpen = computed(() => route.name === "settings");

const activeStepIndex = computed(() => {
  if (state.step.value === "artwork") return 1;
  if (state.step.value === "review") return 2;
  return 0;
});

function toggleSettings() {
  router.push(settingsOpen.value ? "/" : "/settings");
}

async function goToStepIndex(index: number) {
  if (index === 0) {
    state.step.value = "sources";
    return;
  }

  if (index === 1) {
    if (state.selectedCandidates.value.length > 0) {
      state.step.value = "artwork";
    }
    return;
  }

  if (index === 2 && state.selectedCandidates.value.length > 0) {
    if (state.previewPlan.value) {
      state.step.value = "review";
      return;
    }
    await reviewPlan.createPreview();
  }
}
</script>

<template>
  <AppFrame
    :active-step="activeStepIndex"
    :error="task.error.value"
    :settings-open="settingsOpen"
    @select-step="goToStepIndex"
    @toggle-settings="toggleSettings"
  >
    <RouterView />
  </AppFrame>
</template>
