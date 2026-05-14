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
  if (state.step.value === "sources") return 1;
  if (state.step.value === "artwork") return 2;
  if (state.step.value === "review") return 3;
  return 0;
});

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

const scanDisabled = computed(() =>
  task.loading.value || !state.selectedUser.value || state.scanPhase.value === "scanning"
);

const showActionBar = computed(() => !settingsOpen.value);

function toggleSettings() {
  router.push(settingsOpen.value ? "/" : "/settings");
}

async function goToStepIndex(index: number) {
  if (index === 0) {
    state.step.value = "start";
    return;
  }

  if (index === 1) {
    if (state.scanPhase.value === "done") {
      state.step.value = "sources";
    }
    return;
  }

  if (index === 2) {
    if (state.selectedCandidates.value.length > 0) {
      state.step.value = "artwork";
    }
    return;
  }

  if (index === 3 && state.selectedCandidates.value.length > 0) {
    if (state.previewPlan.value) {
      state.step.value = "review";
      return;
    }
    await reviewPlan.createPreview();
  }
}

function goBack() {
  if (state.step.value === "sources") {
    state.step.value = "start";
  } else if (state.step.value === "artwork") {
    state.step.value = "sources";
  } else if (state.step.value === "review") {
    state.step.value = "artwork";
  }
}

async function doScan() {
  await scan();
  if (state.scanPhase.value === "done") {
    state.step.value = "sources";
  }
}

function continueToSources() {
  state.step.value = "sources";
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
          <UiButton v-if="state.step.value !== 'start'" variant="ghost" @click="goBack">
            Back
          </UiButton>

          <!-- Start view: scan button, plus continue when results exist -->
          <template v-if="state.step.value === 'start'">
            <UiButton
              v-if="state.scanPhase.value === 'done'"
              variant="ghost"
              :disabled="scanDisabled"
              @click="doScan"
            >
              Scan again
              <template #icon><Search :size="16" /></template>
            </UiButton>
            <UiButton v-else :disabled="scanDisabled" @click="doScan">
              Scan
              <template #icon><Search :size="16" /></template>
            </UiButton>
            <UiButton v-if="state.scanPhase.value === 'done'" @click="continueToSources">
              Continue
              <template #icon><ArrowRight :size="16" /></template>
            </UiButton>
          </template>

          <!-- All other steps -->
          <UiButton v-else :disabled="nextDisabled" @click="goNext">
            {{ nextLabel }}
            <template #icon>
              <Check v-if="state.step.value === 'review'" :size="18" />
              <ArrowRight v-else :size="16" />
            </template>
          </UiButton>
        </div>
      </div>
    </template>
  </AppShell>
</template>
