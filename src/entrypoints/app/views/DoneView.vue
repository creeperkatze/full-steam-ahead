<script setup lang="ts">
import { Check, Loader2 } from "@lucide/vue";
import type { ApplyProgressEvent, ApplyResult } from "../../../types";

defineProps<{
  applyResult: ApplyResult | null;
  applyProgress: ApplyProgressEvent | null;
}>();
</script>

<template>
  <!-- Apply progress -->
  <section
    v-if="applyProgress"
    class="grid gap-3 rounded-xl border border-accent/30 bg-accent-bg p-5"
  >
    <div class="flex items-center gap-3">
      <Loader2 :size="18" class="shrink-0 animate-spin text-accent" />
      <strong class="min-w-0 flex-1 truncate">{{ applyProgress.step }}</strong>
      <span class="shrink-0 text-xs text-secondary">{{ applyProgress.current }} / {{ applyProgress.total }}</span>
    </div>
    <div class="h-2 overflow-hidden rounded-full bg-surface-3">
      <div
        class="h-full rounded-full bg-accent transition-all duration-300"
        :style="{ width: `${(applyProgress.current / applyProgress.total) * 100}%` }"
      />
    </div>
  </section>

  <!-- Success state -->
  <section v-else-if="applyResult" class="grid gap-4">
    <div class="flex flex-col items-center gap-5 rounded-xl border border-accent/30 bg-accent-bg py-14 text-center">
      <div class="grid size-14 place-items-center rounded-full bg-accent text-accent-contrast">
        <Check :size="28" />
      </div>
      <div>
        <h1 class="text-2xl font-bold">All done!</h1>
        <p class="mt-1 text-secondary">Steam shortcuts and artwork have been updated.</p>
      </div>
      <div class="flex items-center gap-6 rounded-lg border border-border bg-surface-3 px-6 py-3">
        <div class="text-center">
          <strong class="block text-2xl">{{ applyResult.appliedChanges.length }}</strong>
          <span class="text-xs text-secondary">changes applied</span>
        </div>
        <div class="h-10 w-px bg-border" />
        <div class="text-center">
          <strong class="block text-2xl">{{ applyResult.backupsCreated.length }}</strong>
          <span class="text-xs text-secondary">backups created</span>
        </div>
      </div>
    </div>
  </section>

  <!-- Initial loading state -->
  <section v-else class="grid min-h-55 place-items-center rounded-lg border border-border bg-surface-3 p-6 text-secondary">
    <div class="flex items-center gap-3">
      <Loader2 :size="18" class="animate-spin text-accent" />
      <span>Starting...</span>
    </div>
  </section>
</template>
