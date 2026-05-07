<script setup lang="ts">
import { computed } from "vue";
import { FolderPlus, Plus, RefreshCw, Search } from "@lucide/vue";
import UiButton from "../../../components/ui/Button.vue";
import { importSourceName } from "../../../helpers/sourceNames";
import type { ImportCandidate, SteamInstallation, SteamUser } from "../../../types/steam";

const props = defineProps<{
  install: SteamInstallation | null;
  selectedUserId: string;
  selectedUser?: SteamUser;
  candidates: ImportCandidate[];
  selectedIds: Set<string>;
  manualPath: string;
  manualName: string;
  includePlaynite: boolean;
  includeEpic: boolean;
  loading: boolean;
}>();

const emit = defineEmits<{
  "update:selectedUserId": [value: string];
  "update:manualPath": [value: string];
  "update:manualName": [value: string];
  "update:includePlaynite": [value: boolean];
  "update:includeEpic": [value: boolean];
  "refresh-steam": [];
  scan: [];
  "pick-executable": [];
  "add-manual": [];
  "toggle-candidate": [id: string];
  "select-all": [];
  "select-none": [];
}>();

type PlatformKey = "epic" | "playnite" | "manual" | "gog";

interface PlatformCard {
  key: PlatformKey;
  title: string;
  eyebrow: string;
  description: string;
  enabled: boolean;
  candidates: ImportCandidate[];
  selectable: boolean;
}

const selectedCount = computed(() => props.selectedIds.size);

const platformCards = computed<PlatformCard[]>(() => {
  const cards: PlatformCard[] = [
    {
      key: "epic",
      title: importSourceName("epic"),
      eyebrow: "Launcher",
      description: "Installed titles from Epic launcher manifests.",
      enabled: props.includeEpic,
      candidates: candidatesFor("epic"),
      selectable: true
    },
    {
      key: "playnite",
      title: importSourceName("playnite"),
      eyebrow: "Library manager",
      description: "Games from the local Playnite library.",
      enabled: props.includePlaynite,
      candidates: candidatesFor("playnite"),
      selectable: true
    },
    {
      key: "gog",
      title: importSourceName("gog"),
      eyebrow: "Library",
      description: "GOG entries found during source scans.",
      enabled: candidatesFor("gog").length > 0,
      candidates: candidatesFor("gog"),
      selectable: false
    }
  ];
  return cards.filter((card) => card.candidates.length > 0);
});

const manualCandidates = computed(() => candidatesFor("manual"));
const otherCards = computed(() => {
  const grouped = new Map<string, ImportCandidate[]>();
  for (const candidate of props.candidates) {
    if (typeof candidate.source !== "string") {
      const label = candidate.source.other;
      grouped.set(label, [...(grouped.get(label) ?? []), candidate]);
    }
  }
  return Array.from(grouped.entries()).map(([title, candidates]) => ({
    title,
    candidates
  }));
});

function candidatesFor(source: PlatformKey) {
  return props.candidates.filter((candidate) => candidate.source === source);
}

function selectedIn(candidates: ImportCandidate[]) {
  return candidates.filter((candidate) => props.selectedIds.has(candidate.id)).length;
}

function allSelected(candidates: ImportCandidate[]) {
  return candidates.length > 0 && selectedIn(candidates) === candidates.length;
}

function cardEnabled(card: PlatformCard) {
  return card.enabled || card.candidates.length > 0;
}

function setPlatformEnabled(card: PlatformCard, value: boolean) {
  if (card.key === "epic") emit("update:includeEpic", value);
  if (card.key === "playnite") emit("update:includePlaynite", value);
  setCandidatesSelected(card.candidates, value);
}

function setCandidatesSelected(candidates: ImportCandidate[], value: boolean) {
  for (const candidate of candidates) {
    if (props.selectedIds.has(candidate.id) !== value) {
      emit("toggle-candidate", candidate.id);
    }
  }
}

</script>

<template>
  <div class="grid gap-4">
    <section class="flex items-center justify-between gap-4 rounded-lg border border-border bg-surface-3 p-4">
      <div>
        <h2 class="text-base font-semibold">Platform Libraries</h2>
        <p class="text-secondary">{{ candidates.length }} games available / {{ selectedCount }} selected</p>
      </div>
      <div class="flex gap-2">
        <UiButton variant="ghost" :disabled="candidates.length === 0" @click="$emit('select-all')">All</UiButton>
        <UiButton variant="ghost" :disabled="candidates.length === 0" @click="$emit('select-none')">None</UiButton>
        <UiButton variant="secondary" :disabled="loading || !selectedUser" @click="$emit('scan')">
          Scan
          <template #icon>
            <Search :size="16" />
          </template>
        </UiButton>
      </div>
    </section>

    <section class="grid grid-cols-2 gap-4">
      <article
        v-for="card in platformCards"
        :key="card.key"
        class="overflow-hidden rounded-lg border bg-surface-3"
        :class="cardEnabled(card) ? 'border-border' : 'border-border-muted opacity-70'"
      >
        <header class="flex min-h-[88px] items-start justify-between gap-3 border-b border-border bg-surface-4 p-4">
          <label class="flex min-w-0 flex-1 cursor-pointer items-start gap-3">
            <input
              class="mt-1"
              type="checkbox"
              :checked="card.enabled && (card.candidates.length === 0 || allSelected(card.candidates))"
              @change="setPlatformEnabled(card, ($event.target as HTMLInputElement).checked)"
            />
            <span class="min-w-0">
              <span class="mb-1 block text-xs uppercase text-secondary">{{ card.eyebrow }}</span>
              <strong class="block text-lg">{{ card.title }}</strong>
              <span class="block text-secondary">{{ card.description }}</span>
            </span>
          </label>
          <span class="shrink-0 rounded-full border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedIn(card.candidates) }} / {{ card.candidates.length }}
          </span>
        </header>

        <div class="grid max-h-[320px] gap-2 overflow-auto p-3">
          <label
            v-for="candidate in card.candidates"
            :key="candidate.id"
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 rounded-md border border-border bg-surface-5 p-3 transition-colors hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="selectedIds.has(candidate.id)"
              @change="$emit('toggle-candidate', candidate.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate">{{ candidate.name }}</strong>
              <small class="path-cell block">{{ candidate.executablePath }}</small>
              <small v-if="candidate.launchOptions" class="block text-accent">Uses launcher URL</small>
            </span>
          </label>

          <div v-if="card.candidates.length === 0" class="grid min-h-[132px] place-items-center rounded-md border border-dashed border-border-dashed bg-surface-5 p-4 text-center text-secondary">
            Scan to fill this platform.
          </div>
        </div>
      </article>

      <article
        v-for="card in otherCards"
        :key="card.title"
        class="overflow-hidden rounded-lg border border-border bg-surface-3"
      >
        <header class="flex min-h-[88px] items-start justify-between gap-3 border-b border-border bg-surface-4 p-4">
          <label class="flex min-w-0 flex-1 cursor-pointer items-start gap-3">
            <input
              class="mt-1"
              type="checkbox"
              :checked="allSelected(card.candidates)"
              @change="setCandidatesSelected(card.candidates, ($event.target as HTMLInputElement).checked)"
            />
            <span class="min-w-0">
              <span class="mb-1 block text-xs uppercase text-secondary">Custom source</span>
              <strong class="block text-lg">{{ card.title }}</strong>
              <span class="block text-secondary">Games reported by {{ card.title }}.</span>
            </span>
          </label>
          <span class="shrink-0 rounded-full border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedIn(card.candidates) }} / {{ card.candidates.length }}
          </span>
        </header>

        <div class="grid max-h-[320px] gap-2 overflow-auto p-3">
          <label
            v-for="candidate in card.candidates"
            :key="candidate.id"
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 rounded-md border border-border bg-surface-5 p-3 transition-colors hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="selectedIds.has(candidate.id)"
              @change="$emit('toggle-candidate', candidate.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate">{{ candidate.name }}</strong>
              <small class="path-cell block">{{ candidate.executablePath }}</small>
              <small class="block text-secondary">{{ importSourceName(candidate.source) }}</small>
            </span>
          </label>
        </div>
      </article>
    </section>

    <section class="overflow-hidden rounded-lg border border-border bg-surface-3">
      <header class="flex items-start justify-between gap-3 border-b border-border bg-surface-4 p-4">
        <label class="flex min-w-0 flex-1 cursor-pointer items-start gap-3">
          <input
            class="mt-1"
            type="checkbox"
            :checked="allSelected(manualCandidates)"
            :disabled="manualCandidates.length === 0"
            @change="setCandidatesSelected(manualCandidates, ($event.target as HTMLInputElement).checked)"
          />
          <span class="min-w-0">
            <span class="mb-1 block text-xs uppercase text-secondary">Manual</span>
            <strong class="block text-lg">{{ importSourceName("manual") }}</strong>
            <span class="block text-secondary">Add executables directly to this list.</span>
          </span>
        </label>
        <span class="shrink-0 rounded-full border border-border px-2 py-1 text-xs text-secondary">
          {{ selectedIn(manualCandidates) }} / {{ manualCandidates.length }}
        </span>
      </header>

      <div class="grid gap-3 p-3">
        <div class="flex items-center gap-2 rounded-md border border-border bg-surface-5 p-2">
          <UiButton size="icon" variant="secondary" title="Pick executable" @click="$emit('pick-executable')">
            <FolderPlus :size="18" />
          </UiButton>
          <input
            class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-3 px-2 text-primary"
            :value="manualPath"
            placeholder="Executable path"
            @input="$emit('update:manualPath', ($event.target as HTMLInputElement).value)"
          />
          <input
            class="h-9 w-64 rounded-md border border-border bg-surface-3 px-2 text-primary"
            :value="manualName"
            placeholder="Display name"
            @input="$emit('update:manualName', ($event.target as HTMLInputElement).value)"
          />
          <UiButton variant="secondary" :disabled="!manualPath" @click="$emit('add-manual')">
            Add
            <template #icon>
              <Plus :size="16" />
            </template>
          </UiButton>
        </div>

        <div class="grid gap-2">
          <label
            v-for="candidate in manualCandidates"
            :key="candidate.id"
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 rounded-md border border-border bg-surface-5 p-3 transition-colors hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="selectedIds.has(candidate.id)"
              @change="$emit('toggle-candidate', candidate.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate">{{ candidate.name }}</strong>
              <small class="path-cell block">{{ candidate.executablePath }}</small>
            </span>
          </label>

          <div v-if="manualCandidates.length === 0" class="grid min-h-[88px] place-items-center rounded-md border border-dashed border-border-dashed bg-surface-5 p-4 text-center text-secondary">
            No manual games added yet.
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
