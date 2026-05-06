<script setup lang="ts">
import { ArrowLeft } from "@lucide/vue";
import { RouterLink } from "vue-router";
import UiButton from "../components/ui/Button.vue";
import { importSourceName } from "../domain/sourceNames";
import { useAppState } from "../state/appState";

const state = useAppState();
</script>

<template>
  <section class="mx-auto grid max-w-5xl gap-3">
    <div class="flex items-center justify-between rounded-lg border border-border bg-surface-3 p-4">
      <div>
        <h1 class="text-[26px] font-bold leading-tight">Settings</h1>
        <p class="text-secondary">Import behavior, collections, and Steam process options.</p>
      </div>
      <RouterLink to="/">
        <UiButton variant="ghost">
          <ArrowLeft :size="16" />
          Import Flow
        </UiButton>
      </RouterLink>
    </div>

    <div class="grid grid-cols-[minmax(0,1fr)_minmax(320px,0.8fr)] gap-3">
      <section class="rounded-lg border border-border bg-surface-3 p-4">
        <h2 class="mb-3 text-base font-semibold">Apply Options</h2>
        <div class="grid gap-2">
          <label class="flex min-h-10 items-center gap-2 rounded-md border border-border bg-surface-5 px-3">
            <input v-model="state.options.value.writeCollections" type="checkbox" />
            Create managed source collections
          </label>
          <label class="flex min-h-10 items-center gap-2 rounded-md border border-border bg-surface-5 px-3">
            <input v-model="state.options.value.useLegacyCollectionsFallback" type="checkbox" />
            Legacy LevelDB fallback
          </label>
          <label class="flex min-h-10 items-center gap-2 rounded-md border border-danger-border bg-surface-5 px-3 text-danger">
            <input v-model="state.options.value.stopSteam" type="checkbox" />
            Stop Steam before applying
          </label>
          <label class="flex min-h-10 items-center gap-2 rounded-md border border-danger-border bg-surface-5 px-3 text-danger">
            <input v-model="state.options.value.restartSteam" type="checkbox" />
            Restart Steam after applying
          </label>
        </div>
      </section>

      <section class="rounded-lg border border-border bg-surface-3 p-4">
        <h2 class="mb-3 text-base font-semibold">Collections</h2>
        <p class="path-cell mb-3">{{ state.selectedUser.value?.collectionsPath || "No Steam user selected" }}</p>
        <div class="grid gap-2">
          <p
            v-for="source in Array.from(new Set(state.selectedCandidates.value.map((candidate) => importSourceName(candidate.source))))"
            :key="source"
            class="rounded-md border border-border bg-surface-5 px-3 py-2"
          >
            {{ source }}
          </p>
          <p v-if="state.selectedCandidates.value.length === 0" class="text-secondary">
            Select imports to preview managed collections.
          </p>
        </div>
      </section>
    </div>
  </section>
</template>
