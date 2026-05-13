<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { Check, FolderPlus, Loader2, Plus, RefreshCw, Search } from "@lucide/vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import SourceCard from "../../../components/SourceCard.vue";
import UiButton from "../../../components/ui/Button.vue";
import { useAppState } from "../../../composables/useAppState";
import { useTaskStatus } from "../../../composables/useTaskStatus";
import { api } from "../../../helpers/api";
import { importSourceName } from "../../../helpers/sourceNames";
import type { ImportCandidate, ImportSource, ScanProgressEvent } from "../../../types";

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

interface SourceState {
  key: ScannableSource;
  name: string;
  status: "pending" | "scanning" | "done";
  found: number;
}

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

const sourceStates = ref<SourceState[]>(makeSourceStates());
let unlistenScan: (() => void) | undefined;

const selectedCount = computed(() => state.selectedCandidateIds.value.size);
const steamUsers = computed(() =>
  [...(state.install.value?.users ?? [])].sort((a, b) =>
    steamUserName(a).localeCompare(steamUserName(b))
  )
);

const platformCards = computed<PlatformCard[]>(() =>
  SCANNABLE_SOURCES.map(source => ({
    key: source,
    title: importSourceName(source),
    candidates: candidatesFor(source)
  })).filter(card => card.candidates.length > 0)
);

const manualCandidates = computed(() => candidatesFor("manual"));
const otherCards = computed(() => {
  const grouped = new Map<string, ImportCandidate[]>();
  for (const candidate of state.candidates.value) {
    if (typeof candidate.source !== "string") {
      const label = candidate.source.other;
      grouped.set(label, [...(grouped.get(label) ?? []), candidate]);
    }
  }
  return Array.from(grouped.entries()).map(([title, candidates]) => ({ title, candidates }));
});

const doneCount = computed(() => sourceStates.value.filter(s => s.status === "done").length);
const foundTotal = computed(() =>
  sourceStates.value.reduce((sum, s) => sum + (s.status === "done" ? s.found : 0), 0)
);
const scanProgressPct = computed(() =>
  SCANNABLE_SOURCES.length > 0 ? (doneCount.value / SCANNABLE_SOURCES.length) * 100 : 0
);

onMounted(async () => {
  if (!state.install.value) {
    await refreshSteam();
  }
});

onUnmounted(() => {
  unlistenScan?.();
});

function makeSourceStates(): SourceState[] {
  return SCANNABLE_SOURCES.map(key => ({
    key,
    name: importSourceName(key),
    status: "pending" as const,
    found: 0
  }));
}

function candidatesFor(source: ImportSource) {
  return state.candidates.value.filter(candidate => candidate.source === source);
}

function steamUserName(user: { accountName?: string | null }) {
  return user.accountName?.trim() || "Unnamed Steam User";
}

function selectedIn(candidates: ImportCandidate[]) {
  return candidates.filter(c => state.selectedCandidateIds.value.has(c.id)).length;
}

function allSelected(candidates: ImportCandidate[]) {
  return candidates.length > 0 && selectedIn(candidates) === candidates.length;
}

function setCandidatesSelected(candidates: ImportCandidate[], value: boolean) {
  for (const candidate of candidates) {
    if (state.selectedCandidateIds.value.has(candidate.id) !== value) {
      toggleCandidate(candidate.id);
    }
  }
}

async function refreshSteam() {
  const detected = await task.runTask("Detecting Steam", () => api.detectSteam());
  if (!detected) return;

  state.install.value = detected;
  state.selectedUserId.value = detected.users[0]?.steamId ?? "";
  state.invalidatePreview();
}

async function scan() {
  if (!state.selectedUserId.value) return;

  sourceStates.value = makeSourceStates();
  state.scanPhase.value = "scanning";

  unlistenScan?.();
  unlistenScan = await listen<ScanProgressEvent>("scan-progress", (event) => {
    const { source, status, found } = event.payload;
    const entry = sourceStates.value.find(s => s.name === source);
    if (entry) {
      if (status === "scanning") {
        entry.status = "scanning";
      } else if (status === "done") {
        entry.status = "done";
        entry.found = found;
      }
    }
  });

  const found = await task.runTask("Scanning sources", () =>
    api.scanSources({ userSteamId: state.selectedUserId.value, includeSources: [] })
  );

  unlistenScan();
  unlistenScan = undefined;

  if (found !== undefined) {
    state.candidates.value = mergeCandidates(state.candidates.value, found);
    state.selectedCandidateIds.value = new Set(state.candidates.value.map(c => c.id));
    state.invalidatePreview();
    state.scanPhase.value = "done";
  } else {
    state.scanPhase.value = "idle";
  }
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
  state.selectedCandidateIds.value = new Set(state.candidates.value.map(c => c.id));
  state.invalidatePreview();
}

function selectNone() {
  state.selectedCandidateIds.value = new Set();
  state.invalidatePreview();
}

function mergeCandidates(existing: ImportCandidate[], incoming: ImportCandidate[]) {
  const map = new Map(existing.map(c => [c.id, c]));
  for (const candidate of incoming) {
    map.set(candidate.id, candidate);
  }
  return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name));
}
</script>

<template>
  <div class="grid gap-4">

    <!-- ── Welcome (idle) ──────────────────────────────────────────── -->
    <section
      v-if="state.scanPhase.value === 'idle'"
      class="flex flex-col items-center gap-6 rounded-xl border border-accent/30 bg-accent-bg px-8 py-14 text-center"
    >
      <div class="grid size-16 place-items-center rounded-full bg-accent text-accent-contrast">
        <Search :size="28" />
      </div>

      <div>
        <h1 class="text-2xl font-bold">Find your games</h1>
        <p class="mt-1 text-secondary">Scan your installed launchers to import them into Steam.</p>
      </div>

      <!-- Loading Steam -->
      <div v-if="task.loading.value" class="flex items-center gap-2 text-sm text-secondary">
        <Loader2 :size="14" class="animate-spin" />
        Detecting Steam installation…
      </div>

      <!-- Steam not found -->
      <p v-else-if="!state.install.value" class="text-sm text-danger">
        Steam installation not found.
        <button class="ml-1 underline hover:no-underline" @click="refreshSteam">Try again</button>
      </p>

      <!-- No users -->
      <p v-else-if="steamUsers.length === 0" class="text-sm text-danger">
        No Steam users found.
      </p>

      <!-- Ready -->
      <template v-else>
        <div class="flex items-center gap-3 rounded-lg border border-border bg-surface-3 px-4 py-2.5">
          <span class="text-sm text-secondary">Steam User</span>
          <select
            v-model="state.selectedUserId.value"
            class="h-9 rounded-md border border-border bg-surface-5 px-2 text-sm text-primary"
          >
            <option v-for="user in steamUsers" :key="user.steamId" :value="user.steamId">
              {{ steamUserName(user) }}
            </option>
          </select>
        </div>

        <UiButton variant="primary" :disabled="!state.selectedUser.value" @click="scan">
          Scan for games
          <template #icon><Search :size="16" /></template>
        </UiButton>
      </template>
    </section>

    <!-- ── Scanning (progress) ─────────────────────────────────────── -->
    <section
      v-else-if="state.scanPhase.value === 'scanning'"
      class="grid gap-5 rounded-xl border border-border bg-surface-3 p-6"
    >
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-bold">Scanning for games…</h1>
          <p class="mt-0.5 text-sm text-secondary">
            {{ foundTotal }} game{{ foundTotal !== 1 ? "s" : "" }} found so far
          </p>
        </div>
        <Loader2 :size="22" class="animate-spin text-accent" />
      </div>

      <div class="grid gap-1.5">
        <div
          v-for="s in sourceStates"
          :key="s.key"
          class="flex items-center gap-3 rounded-lg border px-3 py-2 transition-colors"
          :class="
            s.status === 'scanning'
              ? 'border-accent/30 bg-accent-bg'
              : 'border-transparent bg-surface-5'
          "
        >
          <div class="shrink-0">
            <Check v-if="s.status === 'done'" :size="14" class="text-accent" />
            <Loader2 v-else-if="s.status === 'scanning'" :size="14" class="animate-spin text-accent" />
            <div v-else class="size-3.5 rounded-full border border-border-muted" />
          </div>
          <span
            class="flex-1 text-sm"
            :class="s.status === 'pending' ? 'text-secondary' : 'text-primary font-medium'"
          >{{ s.name }}</span>
          <span v-if="s.status === 'done'" class="shrink-0 text-xs text-secondary">
            {{ s.found > 0 ? `${s.found} found` : "none" }}
          </span>
        </div>
      </div>

      <div class="space-y-1.5">
        <div class="h-1.5 overflow-hidden rounded-full bg-surface-5">
          <div
            class="h-full rounded-full bg-accent transition-all duration-500"
            :style="{ width: `${scanProgressPct}%` }"
          />
        </div>
        <p class="text-xs text-secondary">{{ doneCount }} of {{ SCANNABLE_SOURCES.length }} sources scanned</p>
      </div>
    </section>

    <!-- ── Results ─────────────────────────────────────────────────── -->
    <template v-else>
      <!-- Header row -->
      <section class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4 rounded-lg border border-border bg-surface-3 p-3">
        <div class="grid grid-cols-[auto_minmax(260px,1fr)_auto] items-center gap-3">
          <strong class="text-base">Steam User</strong>
          <select
            v-model="state.selectedUserId.value"
            class="h-10 w-fit rounded-md border border-border bg-surface-5 px-2 text-primary"
            :disabled="!state.install.value?.users.length"
            @change="state.invalidatePreview()"
          >
            <option v-for="user in steamUsers" :key="user.steamId" :value="user.steamId">
              {{ steamUserName(user) }}
            </option>
          </select>
          <div class="flex items-center gap-4">
            <span class="text-secondary">Found <strong class="text-primary">{{ state.candidates.value.length }}</strong></span>
            <span class="text-secondary">Selected <strong class="text-primary">{{ selectedCount }}</strong></span>
          </div>
        </div>
        <div class="flex gap-2">
          <UiButton variant="ghost" :disabled="state.candidates.value.length === 0" @click="selectAll">All</UiButton>
          <UiButton variant="ghost" :disabled="state.candidates.value.length === 0" @click="selectNone">None</UiButton>
          <UiButton size="icon" variant="ghost" title="Detect Steam again" :disabled="task.loading.value" @click="refreshSteam">
            <RefreshCw :size="16" />
          </UiButton>
          <UiButton variant="secondary" :disabled="task.loading.value || !state.selectedUser.value" @click="scan">
            Re-scan
            <template #icon><Search :size="16" /></template>
          </UiButton>
        </div>
      </section>

      <!-- Source cards -->
      <section class="grid gap-3">
        <SourceCard
          v-for="card in platformCards"
          :key="card.key"
          :title="card.title"
          :candidates="card.candidates"
          :selected-ids="state.selectedCandidateIds.value"
          @toggle="toggleCandidate"
          @set-all="setCandidatesSelected(card.candidates, $event)"
        />

        <SourceCard
          v-for="card in otherCards"
          :key="card.title"
          :title="card.title"
          :candidates="card.candidates"
          :selected-ids="state.selectedCandidateIds.value"
          show-source
          @toggle="toggleCandidate"
          @set-all="setCandidatesSelected(card.candidates, $event)"
        />
      </section>

      <!-- Manual section -->
      <section class="overflow-hidden rounded-lg border border-border bg-surface-3">
        <header class="flex items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2">
          <label class="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
            <input
              type="checkbox"
              :checked="allSelected(manualCandidates)"
              :disabled="manualCandidates.length === 0"
              @change="setCandidatesSelected(manualCandidates, ($event.target as HTMLInputElement).checked)"
            />
            <strong class="block min-w-0 truncate text-base">{{ importSourceName("manual") }}</strong>
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
              v-model="state.manualPath.value"
              class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-3 px-2 text-primary"
              placeholder="Executable path"
            />
            <input
              v-model="state.manualName.value"
              class="h-9 w-64 rounded-md border border-border bg-surface-3 px-2 text-primary"
              placeholder="Display name"
            />
            <UiButton variant="secondary" :disabled="!state.manualPath.value" @click="addManual">
              Add
              <template #icon><Plus :size="16" /></template>
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

            <div
              v-if="manualCandidates.length === 0"
              class="grid min-h-22 place-items-center rounded-md border border-dashed border-border-dashed bg-surface-5 p-4 text-center text-secondary"
            >
              No manual games added yet.
            </div>
          </div>
        </div>
      </section>
    </template>

  </div>
</template>
