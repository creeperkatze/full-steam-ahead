<script lang="ts">
let sourcesInitialized = false;
</script>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import { FolderPlus, Plus, RefreshCw, Search } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import UiButton from "../../../components/ui/Button.vue";
import { useAppState } from "../../../composables/useAppState";
import { useTaskStatus } from "../../../composables/useTaskStatus";
import { api } from "../../../helpers/api";
import { importSourceName } from "../../../helpers/sourceNames";
import type { ImportCandidate, ImportSource } from "../../../types/steam";

const state = useAppState();
const task = useTaskStatus();

type ScannableSource =
  | "playnite"
  | "epic"
  | "amazon"
  | "gog"
  | "itch"
  | "origin"
  | "ubisoftConnect"
  | "gamePass";

interface PlatformCard {
  key: ScannableSource;
  title: string;
  candidates: ImportCandidate[];
}

const SCANNABLE_SOURCES: ScannableSource[] = [
  "playnite",
  "epic",
  "amazon",
  "gog",
  "itch",
  "origin",
  "ubisoftConnect",
  "gamePass"
];

const selectedCount = computed(() => state.selectedCandidateIds.value.size);
const activeUserLabel = computed(() => {
  const user = state.selectedUser.value;
  if (!user) return "No Steam user";
  return steamUserName(user);
});
const steamUsers = computed(() =>
  [...(state.install.value?.users ?? [])].sort((left, right) =>
    steamUserName(left).localeCompare(steamUserName(right))
  )
);

const platformCards = computed<PlatformCard[]>(() => {
  return SCANNABLE_SOURCES.map((source) => ({
    key: source,
    title: importSourceName(source),
    candidates: candidatesFor(source)
  })).filter((card) => card.candidates.length > 0);
});

const manualCandidates = computed(() => candidatesFor("manual"));
const otherCards = computed(() => {
  const grouped = new Map<string, ImportCandidate[]>();
  for (const candidate of state.candidates.value) {
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

onMounted(() => {
  void initializeSources();
});

function candidatesFor(source: ImportSource) {
  return state.candidates.value.filter((candidate) => candidate.source === source);
}

function steamUserName(user: { accountName?: string | null }) {
  return user.accountName?.trim() || "Unnamed Steam User";
}

function selectedIn(candidates: ImportCandidate[]) {
  return candidates.filter((candidate) => state.selectedCandidateIds.value.has(candidate.id)).length;
}

function allSelected(candidates: ImportCandidate[]) {
  return candidates.length > 0 && selectedIn(candidates) === candidates.length;
}

function setPlatformEnabled(card: PlatformCard, value: boolean) {
  setCandidatesSelected(card.candidates, value);
}

function setCandidatesSelected(candidates: ImportCandidate[], value: boolean) {
  for (const candidate of candidates) {
    if (state.selectedCandidateIds.value.has(candidate.id) !== value) {
      toggleCandidate(candidate.id);
    }
  }
}

async function initializeSources() {
  if (sourcesInitialized) return;
  sourcesInitialized = true;
  await refreshSteam();
}

async function refreshSteam() {
  const detected = await task.runTask("Detecting Steam", () => api.detectSteam());
  if (!detected) return;

  state.install.value = detected;
  state.selectedUserId.value = detected.users[0]?.steamId ?? "";
  state.invalidatePreview();

  if (state.selectedUserId.value) {
    await scan();
  }
}

async function scan() {
  if (!state.selectedUserId.value) return;

  const found = await task.runTask("Scanning sources", () =>
    api.scanSources({
      userSteamId: state.selectedUserId.value,
      includeSources: []
    })
  );
  if (!found) return;

  state.candidates.value = mergeCandidates(state.candidates.value, found);
  state.selectedCandidateIds.value = new Set(state.candidates.value.map((candidate) => candidate.id));
  state.invalidatePreview();
}

async function pickExecutable() {
  const picked = await open({
    multiple: false,
    filters: [{ name: "Executable", extensions: ["exe", "bat", "cmd"] }]
  });
  if (typeof picked === "string") {
    state.manualPath.value = picked;
  }
}

async function addManual() {
  if (!state.selectedUserId.value || !state.manualPath.value.trim()) return;

  const candidate = await task.runTask("Adding manual entry", () =>
    api.createManualCandidate({
      userSteamId: state.selectedUserId.value,
      executablePath: state.manualPath.value.trim(),
      displayName: state.manualName.value.trim() || undefined,
      source: "manual",
      tags: ["Manual"]
    })
  );
  if (!candidate) return;

  state.candidates.value = mergeCandidates(state.candidates.value, [candidate]);
  state.selectedCandidateIds.value = new Set([...state.selectedCandidateIds.value, candidate.id]);
  state.manualPath.value = "";
  state.manualName.value = "";
  state.invalidatePreview();
}

function toggleCandidate(id: string) {
  const next = new Set(state.selectedCandidateIds.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  state.selectedCandidateIds.value = next;
  state.invalidatePreview();
}

function selectAll() {
  state.selectedCandidateIds.value = new Set(state.candidates.value.map((candidate) => candidate.id));
  state.invalidatePreview();
}

function selectNone() {
  state.selectedCandidateIds.value = new Set();
  state.invalidatePreview();
}

function mergeCandidates(existing: ImportCandidate[], incoming: ImportCandidate[]) {
  const map = new Map(existing.map((candidate) => [candidate.id, candidate]));
  for (const candidate of incoming) {
    map.set(candidate.id, candidate);
  }
  return Array.from(map.values()).sort((left, right) => left.name.localeCompare(right.name));
}

</script>

<template>
  <div class="grid gap-4">
    <section class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4 rounded-lg border border-border bg-surface-3 p-3">
      <div class="grid grid-cols-[minmax(180px,0.8fr)_minmax(0,1fr)_auto_auto] items-center gap-3">
        <div class="min-w-0">
          <span class="block text-xs uppercase text-secondary">Steam user</span>
          <strong class="block truncate text-base">{{ activeUserLabel }}</strong>
        </div>
        <select
          v-model="state.selectedUserId.value"
          class="h-9 min-w-0 rounded-md border border-border bg-surface-5 px-2 text-primary"
          :disabled="!state.install.value?.users.length"
          @change="state.invalidatePreview()"
        >
          <option
          v-for="user in steamUsers"
          :key="user.steamId"
          :value="user.steamId"
        >
            {{ steamUserName(user) }}
          </option>
        </select>
        <div class="rounded-md border border-border bg-surface-5 px-3 py-1.5 text-right">
          <span class="block text-xs uppercase text-secondary">Found</span>
          <strong>{{ state.candidates.value.length }}</strong>
        </div>
        <div class="rounded-md border border-border bg-surface-5 px-3 py-1.5 text-right">
          <span class="block text-xs uppercase text-secondary">Selected</span>
          <strong>{{ selectedCount }}</strong>
        </div>
      </div>
      <div class="flex gap-2">
        <UiButton variant="ghost" :disabled="state.candidates.value.length === 0" @click="selectAll">All</UiButton>
        <UiButton variant="ghost" :disabled="state.candidates.value.length === 0" @click="selectNone">None</UiButton>
        <UiButton size="icon" variant="ghost" title="Detect Steam again" :disabled="task.loading.value" @click="refreshSteam">
          <RefreshCw :size="16" />
        </UiButton>
        <UiButton variant="secondary" :disabled="task.loading.value || !state.selectedUser.value" @click="scan">
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
        class="overflow-hidden rounded-lg border border-border bg-surface-3"
      >
        <header class="flex min-h-12 items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2">
          <label class="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
            <input
              type="checkbox"
              :checked="allSelected(card.candidates)"
              @change="setPlatformEnabled(card, ($event.target as HTMLInputElement).checked)"
            />
            <span class="min-w-0">
              <strong class="block truncate text-base">{{ card.title }}</strong>
            </span>
          </label>
          <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedIn(card.candidates) }} / {{ card.candidates.length }}
          </span>
        </header>

        <div class="grid max-h-80 overflow-auto">
          <label
            v-for="candidate in card.candidates"
            :key="candidate.id"
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 border-b border-border-muted px-3 py-2.5 transition-colors last:border-b-0 hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="state.selectedCandidateIds.value.has(candidate.id)"
              @change="toggleCandidate(candidate.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate">{{ candidate.name }}</strong>
              <small class="path-cell block">{{ candidate.executablePath }}</small>
              <small v-if="candidate.launchOptions" class="block text-accent">Uses launcher URL</small>
            </span>
          </label>

          <div v-if="card.candidates.length === 0" class="grid min-h-33 place-items-center border border-dashed border-border-dashed bg-surface-5 p-4 text-center text-secondary">
            Scan to fill this platform.
          </div>
        </div>
      </article>

      <article
        v-for="card in otherCards"
        :key="card.title"
        class="overflow-hidden rounded-lg border border-border bg-surface-3"
      >
        <header class="flex min-h-12 items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2">
          <label class="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
            <input
              type="checkbox"
              :checked="allSelected(card.candidates)"
              @change="setCandidatesSelected(card.candidates, ($event.target as HTMLInputElement).checked)"
            />
            <span class="min-w-0">
              <strong class="block truncate text-base">{{ card.title }}</strong>
            </span>
          </label>
          <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedIn(card.candidates) }} / {{ card.candidates.length }}
          </span>
        </header>

        <div class="grid max-h-80 overflow-auto">
          <label
            v-for="candidate in card.candidates"
            :key="candidate.id"
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 border-b border-border-muted px-3 py-2.5 transition-colors last:border-b-0 hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="state.selectedCandidateIds.value.has(candidate.id)"
              @change="toggleCandidate(candidate.id)"
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
      <header class="flex items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2">
        <label class="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            :checked="allSelected(manualCandidates)"
            :disabled="manualCandidates.length === 0"
            @change="setCandidatesSelected(manualCandidates, ($event.target as HTMLInputElement).checked)"
          />
          <span class="min-w-0">
            <strong class="block truncate text-base">{{ importSourceName("manual") }}</strong>
          </span>
        </label>
        <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
          {{ selectedIn(manualCandidates) }} / {{ manualCandidates.length }}
        </span>
      </header>

      <div class="grid gap-3 p-3">
        <div class="flex items-center gap-2 rounded-md border border-border bg-surface-5 p-2">
          <UiButton size="icon" variant="secondary" title="Pick executable" @click="pickExecutable">
            <FolderPlus :size="18" />
          </UiButton>
          <input
            class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-3 px-2 text-primary"
            v-model="state.manualPath.value"
            placeholder="Executable path"
          />
          <input
            class="h-9 w-64 rounded-md border border-border bg-surface-3 px-2 text-primary"
            v-model="state.manualName.value"
            placeholder="Display name"
          />
          <UiButton variant="secondary" :disabled="!state.manualPath.value" @click="addManual">
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
            class="grid cursor-pointer grid-cols-[auto_1fr] gap-x-3 rounded-md border border-border bg-surface-5 px-3 py-2.5 transition-colors hover:bg-surface-hover"
          >
            <input
              class="mt-1"
              type="checkbox"
              :checked="state.selectedCandidateIds.value.has(candidate.id)"
              @change="toggleCandidate(candidate.id)"
            />
            <span class="min-w-0">
              <strong class="block truncate">{{ candidate.name }}</strong>
              <small class="path-cell block">{{ candidate.executablePath }}</small>
            </span>
          </label>

          <div v-if="manualCandidates.length === 0" class="grid min-h-22 place-items-center rounded-md border border-dashed border-border-dashed bg-surface-5 p-4 text-center text-secondary">
            No manual games added yet.
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
