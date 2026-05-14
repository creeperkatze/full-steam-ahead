<script setup lang="ts">
import { useAppState } from "../../../composables/useAppState";
import { useReviewPlan } from "../../../composables/useReviewPlan";
import ArtworkView from "./ArtworkView.vue";
import DoneView from "./DoneView.vue";
import ReviewView from "./ReviewView.vue";
import SourcesView from "./SourcesView.vue";
import StartView from "./StartView.vue";

const state = useAppState();
const reviewPlan = useReviewPlan();
</script>

<template>
  <div class="flex flex-1 flex-col gap-3">
    <StartView v-if="state.step.value === 'start'" />

    <SourcesView v-else-if="state.step.value === 'sources'" />

    <ArtworkView v-else-if="state.step.value === 'artwork'" />

    <ReviewView
      v-else-if="state.step.value === 'review'"
      :plan="state.previewPlan.value"
    />

    <DoneView
      v-else
      :apply-result="state.applyResult.value"
      :apply-progress="reviewPlan.applyProgress.value"
    />
  </div>
</template>
