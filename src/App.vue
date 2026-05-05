<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter, useRoute, RouterView } from "vue-router";
import AppFrame from "./components/AppFrame.vue";
import { useAppState } from "./state/appState";

const router = useRouter();
const route = useRoute();
const state = useAppState();
const settingsOpen = computed(() => route.name === "settings");

onMounted(() => {
  state.initialize();
});

function toggleSettings() {
  router.push(settingsOpen.value ? "/" : "/settings");
}
</script>

<template>
  <AppFrame
    :active-step="state.activeStepIndex.value"
    :error="state.error.value"
    :settings-open="settingsOpen"
    @toggle-settings="toggleSettings"
  >
    <RouterView />
  </AppFrame>
</template>
