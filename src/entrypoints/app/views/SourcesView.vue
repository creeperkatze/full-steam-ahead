<script setup lang="ts">
import { Check, FolderPlus, Loader2, Plus, RefreshCw, Search } from "@lucide/vue";
import { computed, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import SourceCard from "../../../components/SourceCard.vue";
import SourceIcon from "../../../components/SourceIcon.vue";
import ItemRow from "../../../components/ui/ItemRow.vue";
import UiButton from "../../../components/ui/Button.vue";
import { useAppState } from "../../../composables/useAppState";
import { useTaskStatus } from "../../../composables/useTaskStatus";
import { useScanSources, SCANNABLE_SOURCES } from "../../../composables/useScanSources";
import { api } from "../../../helpers/api";
import { importSourceName } from "../../../helpers/sourceNames";
import type { ImportCandidate, ImportSource } from "../../../types";

const state = useAppState();
const task = useTaskStatus();
const { sourceStates } = useScanSources();

interface PlatformCard {
  key: string;
  title: string;
  candidates: ImportCandidate[];
}

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

  state.candidates.value = [...state.candidates.value, candidate].sort((a, b) =>
    a.name.localeCompare(b.name)
  );
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
</script>

<template>
  <div class="flex flex-1 flex-col gap-4">

    <!-- ── Welcome (idle) ──────────────────────────────────────────── -->
    <section
      v-if="state.scanPhase.value === 'idle'"
      class="flex flex-1 flex-col items-center justify-center gap-6 rounded-xl border border-accent/30 bg-accent-bg px-8 py-8 text-center"
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

      <!-- Ready: just the user selector, scan button is in the footer -->
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
      </template>
    </section>

    <!-- ── Scanning (progress) ─────────────────────────────────────── -->
    <section
      v-else-if="state.scanPhase.value === 'scanning'"
      class="overflow-hidden rounded-xl border border-border"
    >
      <div class="flex items-center justify-between border-b border-border bg-surface-4 px-3 py-2.5">
        <div>
          <h1 class="text-base font-bold">Scanning for games…</h1>
          <p class="text-sm text-secondary">
            {{ foundTotal }} game{{ foundTotal !== 1 ? "s" : "" }} found so far
          </p>
        </div>
        <Loader2 :size="20" class="animate-spin text-accent" />
      </div>

      <div class="grid gap-1.5 bg-surface-3 p-2">
        <ItemRow
          v-for="s in sourceStates"
          :key="s.key"
          :active="s.status === 'scanning'"
        >
          <template #leading>
            <Check v-if="s.status === 'done'" :size="14" class="shrink-0 text-accent" />
            <Loader2 v-else-if="s.status === 'scanning'" :size="14" class="shrink-0 animate-spin text-accent" />
            <div v-else class="size-3.5 shrink-0 rounded-full border border-border-muted" />
            <SourceIcon :source="s.key" class="size-4 shrink-0" />
          </template>

          <span :class="s.status === 'pending' ? 'text-secondary' : 'font-medium'">{{ s.name }}</span>

          <template #trailing>
            <span v-if="s.status === 'done'" class="shrink-0 text-xs text-secondary">
              {{ s.found > 0 ? `${s.found} found` : "none" }}
            </span>
          </template>
        </ItemRow>

        <div class="space-y-1.5 px-1 pb-1 pt-0.5">
          <div class="h-1.5 overflow-hidden rounded-full bg-surface-5">
            <div
              class="h-full rounded-full bg-accent transition-all duration-500"
              :style="{ width: `${scanProgressPct}%` }"
            />
          </div>
          <p class="text-xs text-secondary">{{ doneCount }} of {{ SCANNABLE_SOURCES.length }} sources scanned</p>
        </div>
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
          <UiButton variant="secondary" :disabled="task.loading.value || !state.selectedUser.value" @click="$emit('rescan')">
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
          :source="card.key"
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
      <section class="overflow-hidden rounded-xl border border-border">
        <label class="flex cursor-pointer items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5">
          <input
            type="checkbox"
            :checked="allSelected(manualCandidates)"
            :disabled="manualCandidates.length === 0"
            @change="setCandidatesSelected(manualCandidates, ($event.target as HTMLInputElement).checked)"
          />
          <strong class="min-w-0 flex-1 truncate text-base">{{ importSourceName("manual") }}</strong>
          <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedIn(manualCandidates) }} / {{ manualCandidates.length }}
          </span>
        </label>

        <div class="grid gap-1.5 bg-surface-3 p-2">
          <div class="flex items-center gap-2 rounded-lg border border-border/60 bg-surface-5 px-3 py-2">
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

          <ItemRow
            v-for="candidate in manualCandidates"
            :key="candidate.id"
            as="label"
            interactive
          >
            <template #leading>
              <input
                type="checkbox"
                :checked="state.selectedCandidateIds.value.has(candidate.id)"
                @change="toggleCandidate(candidate.id)"
              />
            </template>
            <strong class="block truncate">{{ candidate.name }}</strong>
            <small class="block text-secondary/70">{{ candidate.executablePath }}</small>
          </ItemRow>

          <div
            v-if="manualCandidates.length === 0"
            class="grid min-h-20 place-items-center rounded-lg border border-dashed border-border-dashed p-4 text-center text-sm text-secondary"
          >
            No manual games added yet.
          </div>
        </div>
      </section>
    </template>

  </div>
</template>
