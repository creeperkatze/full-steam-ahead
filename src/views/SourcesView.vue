<script setup lang="ts">
import { computed } from "vue";
import { FolderPlus, Plus, RefreshCw, Search } from "@lucide/vue";
import UiButton from "../components/ui/UiButton.vue";
import type { ImportCandidate, SteamInstallation, SteamUser } from "../types";

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

defineEmits<{
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

const selectedCount = computed(() => props.selectedIds.size);

function sourceLabel(source: ImportCandidate["source"]) {
  return typeof source === "string" ? source : source.other;
}
</script>

<template>
  <div class="grid gap-3">
    <section class="grid grid-cols-[minmax(0,1.5fr)_minmax(360px,0.9fr)] gap-5 rounded-lg border border-fsa-line bg-fsa-panel p-4">
      <div>
        <span class="mb-2 block text-xs font-bold uppercase text-fsa-accent">Welcome</span>
        <h1 class="mb-2 text-[26px] font-bold leading-tight">Choose what gets imported into Steam.</h1>
        <p class="max-w-3xl text-fsa-muted">
          Scanning reads library metadata only. Nothing is written until the final review step creates backups and applies.
        </p>
      </div>

      <div class="grid gap-2">
        <div class="min-w-0 rounded-md border border-fsa-line bg-fsa-panel-3 p-3">
          <div class="mb-2 flex items-start justify-between gap-3">
            <div class="min-w-0">
              <span class="mb-1 block text-xs uppercase text-fsa-muted">Steam install</span>
              <strong class="mb-1 block text-lg">{{ install ? "Detected" : "Not detected" }}</strong>
              <small class="block truncate text-fsa-muted">{{ install?.installPath || "Refresh once Steam is installed." }}</small>
            </div>
            <UiButton variant="ghost" title="Refresh Steam detection" :disabled="loading" @click="$emit('refresh-steam')">
              <RefreshCw :size="16" />
              Refresh
            </UiButton>
          </div>
        </div>

        <div class="min-w-0 rounded-md border border-fsa-line bg-fsa-panel-3 p-3">
          <span class="mb-2 block text-xs uppercase text-fsa-muted">Steam account</span>
          <select
            class="mb-2 h-9 w-full rounded-md border border-fsa-line bg-fsa-panel px-2 text-fsa-text"
            :value="selectedUserId"
            :disabled="loading || (install?.users.length ?? 0) === 0"
            @change="$emit('update:selectedUserId', ($event.target as HTMLSelectElement).value)"
          >
            <option v-if="(install?.users.length ?? 0) === 0" value="">No Steam users found</option>
            <option v-for="user in install?.users ?? []" :key="user.steamId" :value="user.steamId">
              {{ user.accountName || user.steamId }}
            </option>
          </select>
          <small class="block truncate text-fsa-muted">
            {{ selectedUser?.shortcutsPath || "No shortcuts file selected." }}
          </small>
        </div>
      </div>
    </section>

    <section class="rounded-lg border border-fsa-line bg-fsa-panel p-4">
      <div class="mb-3 flex items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-semibold">Detected Sources</h2>
          <p class="text-fsa-muted">Pick sources, scan them, then choose the games to carry forward.</p>
        </div>
        <UiButton variant="secondary" :disabled="loading || !selectedUser" @click="$emit('scan')">
          <Search :size="16" />
          Scan Sources
        </UiButton>
      </div>

      <div class="mb-3 grid grid-cols-3 gap-2">
        <label class="grid min-h-[88px] grid-cols-[auto_1fr] gap-x-2 gap-y-1 rounded-md border border-fsa-line bg-fsa-panel-3 p-3">
          <input
            type="checkbox"
            :checked="includeEpic"
            @change="$emit('update:includeEpic', ($event.target as HTMLInputElement).checked)"
          />
          <strong>Epic Games</strong>
          <span class="col-start-2 text-fsa-muted">Installed games from Epic launcher manifests.</span>
        </label>
        <label class="grid min-h-[88px] grid-cols-[auto_1fr] gap-x-2 gap-y-1 rounded-md border border-fsa-line bg-fsa-panel-3 p-3">
          <input
            type="checkbox"
            :checked="includePlaynite"
            @change="$emit('update:includePlaynite', ($event.target as HTMLInputElement).checked)"
          />
          <strong>Playnite</strong>
          <span class="col-start-2 text-fsa-muted">Games found in the local Playnite library.</span>
        </label>
        <div class="grid min-h-[88px] grid-cols-[auto_1fr] gap-x-2 gap-y-1 rounded-md border border-fsa-line bg-fsa-panel-3 p-3">
          <FolderPlus :size="18" />
          <strong>Manual executable</strong>
          <span class="col-start-2 text-fsa-muted">Add one game directly from an executable.</span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <UiButton size="icon" variant="secondary" title="Pick executable" @click="$emit('pick-executable')">
          <Plus :size="18" />
        </UiButton>
        <input
          class="h-9 flex-1 rounded-md border border-fsa-line bg-fsa-panel-3 px-2 text-fsa-text"
          :value="manualPath"
          placeholder="Executable path"
          @input="$emit('update:manualPath', ($event.target as HTMLInputElement).value)"
        />
        <input
          class="h-9 w-60 rounded-md border border-fsa-line bg-fsa-panel-3 px-2 text-fsa-text"
          :value="manualName"
          placeholder="Display name"
          @input="$emit('update:manualName', ($event.target as HTMLInputElement).value)"
        />
        <UiButton variant="secondary" :disabled="!manualPath" @click="$emit('add-manual')">Add</UiButton>
      </div>
    </section>

    <section class="overflow-hidden rounded-lg border border-fsa-line bg-fsa-panel">
      <div class="flex items-center justify-between gap-4 border-b border-fsa-line bg-fsa-panel-2 px-4 py-3">
        <div>
          <h2 class="text-base font-semibold">Import Selection</h2>
          <p class="text-fsa-muted">{{ candidates.length }} found / {{ selectedCount }} selected</p>
        </div>
        <div class="flex gap-2">
          <UiButton variant="ghost" :disabled="candidates.length === 0" @click="$emit('select-all')">All</UiButton>
          <UiButton variant="ghost" :disabled="candidates.length === 0" @click="$emit('select-none')">None</UiButton>
        </div>
      </div>

      <table>
        <thead>
          <tr>
            <th class="w-11"></th>
            <th>Name</th>
            <th>Source</th>
            <th>Executable</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="candidate in candidates" :key="candidate.id">
            <td>
              <input
                type="checkbox"
                :checked="selectedIds.has(candidate.id)"
                @change="$emit('toggle-candidate', candidate.id)"
              />
            </td>
            <td>
              <strong>{{ candidate.name }}</strong>
              <small v-if="candidate.launchOptions">Uses launcher URL</small>
            </td>
            <td>{{ sourceLabel(candidate.source) }}</td>
            <td class="path-cell">{{ candidate.executablePath }}</td>
          </tr>
          <tr v-if="candidates.length === 0">
            <td colspan="4" class="h-20 text-center text-fsa-muted">No games scanned yet.</td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>
