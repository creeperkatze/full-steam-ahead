<script setup lang="ts">
import { ArrowRight, Check, Search } from "@lucide/vue";
import { computed } from "vue";
import { useRouter, useRoute, RouterView } from "vue-router";
import AppShell from "../../components/AppShell.vue";
import UiButton from "../../components/ui/Button.vue";
import { useAppState } from "../../composables/useAppState";
import { useReviewPlan } from "../../composables/useReviewPlan";
import { useScanSources } from "../../composables/useScanSources";
import { useTaskStatus } from "../../composables/useTaskStatus";

const router = useRouter();
const route = useRoute();
const state = useAppState();
const reviewPlan = useReviewPlan();
const task = useTaskStatus();
const { scan } = useScanSources();
const settingsOpen = computed(() => route.name === "settings");

const activeStepIndex = computed(() => {
  if (state.step.value === "artwork") return 1;
  if (state.step.value === "review") return 2;
  return 0;
});

const isSourcesIdle = computed(() =>
  state.step.value === "sources" && state.scanPhase.value !== "done"
);

const nextLabel = computed(() => {
  if (isSourcesIdle.value) return "Scan for games";
  if (state.step.value === "review") return state.applyResult.value ? "Applied" : "Apply";
  return "Continue";
});

const nextDisabled = computed(() => {
  if (task.loading.value) return true;
  if (isSourcesIdle.value) return !state.selectedUser.value;
  if (state.step.value === "sources") return state.selectedCandidates.value.length === 0;
  if (state.step.value === "review") return !state.previewPlan.value || !!state.applyResult.value;
  return false;
});

const showActionBar = computed(() => !settingsOpen.value);

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

function goBack() {
  if (state.step.value === "review") {
    state.step.value = "artwork";
  } else if (state.step.value === "artwork") {
    state.step.value = "sources";
  }
}

async function goNext() {
  if (state.step.value === "sources") {
    if (state.scanPhase.value === "done") {
      state.step.value = "artwork";
    } else {
      await scan();
    }
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
  <AppShell
    :active-step="activeStepIndex"
    :error="task.error.value"
    :settings-open="settingsOpen"
    @select-step="goToStepIndex"
    @toggle-settings="toggleSettings"
  >
    <RouterView />

    <template #footer>
      <div v-if="showActionBar" class="flex shrink-0 justify-center px-2">
        <div class="flex items-center gap-2">
          <UiButton v-if="state.step.value !== 'sources'" variant="ghost" @click="goBack">
            Back
          </UiButton>
          <UiButton :disabled="nextDisabled" @click="goNext">
            {{ nextLabel }}
            <template #icon>
              <Check v-if="state.step.value === 'review'" :size="18" />
              <Search v-else-if="isSourcesIdle" :size="16" />
              <ArrowRight v-else :size="16" />
            </template>
          </UiButton>
        </div>
      </div>
    </template>
  </AppShell>
</template>
