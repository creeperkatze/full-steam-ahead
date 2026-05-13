<script setup lang="ts">
import { useAppState } from "../../../composables/useAppState";
import { useReviewPlan } from "../../../composables/useReviewPlan";
import ArtworkView from "./ArtworkView.vue";
import ReviewView from "./ReviewView.vue";
import SourcesView from "./SourcesView.vue";

const state = useAppState();
const reviewPlan = useReviewPlan();
</script>

<template>
  <div class="flex flex-1 flex-col gap-3">
    <SourcesView v-if="state.step.value === 'sources'" />

    <ArtworkView v-else-if="state.step.value === 'artwork'" />

    <ReviewView
      v-else
      :plan="state.previewPlan.value"
      :apply-result="state.applyResult.value"
      :apply-progress="reviewPlan.applyProgress.value"
    />
  </div>
</template>
