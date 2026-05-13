<script setup lang="ts">
import { computed } from "vue";
import { importSourceName } from "../helpers/sourceNames";
import type { ImportCandidate } from "../types/steam";

const props = defineProps<{
  title: string;
  candidates: ImportCandidate[];
  selectedIds: Set<string>;
  showSource?: boolean;
}>();

const emit = defineEmits<{
  toggle: [id: string];
  "set-all": [value: boolean];
}>();

const selectedCount = computed(() =>
  props.candidates.filter(c => props.selectedIds.has(c.id)).length
);

const allSelected = computed(() =>
  props.candidates.length > 0 && selectedCount.value === props.candidates.length
);
</script>

<template>
  <article class="overflow-hidden rounded-lg border border-border bg-surface-3">
    <header class="flex min-h-12 items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2">
      <label class="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
        <input
          type="checkbox"
          :checked="allSelected"
          @change="emit('set-all', ($event.target as HTMLInputElement).checked)"
        />
        <strong class="block min-w-0 truncate text-base">{{ title }}</strong>
      </label>
      <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
        {{ selectedCount }} / {{ candidates.length }}
      </span>
    </header>

    <div class="grid max-h-80 overflow-auto">
      <label
        v-for="candidate in candidates"
        :key="candidate.id"
        class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 border-b border-border-muted px-3 py-2.5 transition-colors last:border-b-0 hover:bg-surface-hover"
      >
        <input
          class="mt-1"
          type="checkbox"
          :checked="selectedIds.has(candidate.id)"
          @change="emit('toggle', candidate.id)"
        />
        <span class="min-w-0">
          <strong class="block truncate">{{ candidate.name }}</strong>
          <small class="path-cell block">{{ candidate.executablePath }}</small>
          <small v-if="candidate.launchOptions && !showSource" class="block text-accent">Uses launcher URL</small>
          <small v-if="showSource" class="block text-secondary">{{ importSourceName(candidate.source) }}</small>
        </span>
      </label>
    </div>
  </article>
</template>
