<script setup lang="ts">
import { useAppState } from "../../../composables/useAppState";

const state = useAppState();
</script>

<template>
  <section class="mx-auto grid max-w-5xl gap-3">
    <section class="rounded-lg border border-border bg-surface-3 p-4">
      <h2 class="mb-3 text-base font-semibold">Apply Options</h2>
      <div class="grid gap-2">
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
  </section>
</template>
