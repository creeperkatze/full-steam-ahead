<script setup lang="ts">
import { AlertTriangle, Check, ChevronDown, FolderArchive, Gamepad2, Image, Library, ListChecks, Loader2, Save } from "@lucide/vue";
import { computed } from "vue";
import type { ApplyProgressEvent, ApplyResult, PlannedChange, PreviewPlan } from "../../../types";

const props = defineProps<{
  plan: PreviewPlan | null;
  applyResult: ApplyResult | null;
  applyProgress: ApplyProgressEvent | null;
}>();

interface ArtworkChange {
  kind: string;
  source: string;
  destructive: boolean;
}

interface GameReview {
  name: string;
  shortcut?: PlannedChange;
  collections: string[];
  artwork: ArtworkChange[];
}

const games = computed(() => {
  const grouped = new Map<string, GameReview>();

  for (const change of props.plan?.changes ?? []) {
    const name = change.gameName;
    if (!name) continue;

    const game = grouped.get(name) ?? { name, collections: [], artwork: [] };

    if (change.kind === "addShortcut" || change.kind === "updateShortcut") {
      game.shortcut = change;
    } else if (change.kind === "updateCollections") {
      game.collections.push(collectionName(change));
    } else if (change.kind === "writeArtwork") {
      game.artwork.push({
        kind: artworkKind(change),
        source: artworkSource(change),
        destructive: change.destructive
      });
    }

    grouped.set(name, game);
  }

  return Array.from(grouped.values()).sort((a, b) => a.name.localeCompare(b.name));
});

const summary = computed(() => {
  const changes = props.plan?.changes ?? [];
  return {
    games: games.value.length,
    shortcuts: changes.filter(c => c.kind === "addShortcut" || c.kind === "updateShortcut").length,
    artwork: changes.filter(c => c.kind === "writeArtwork").length,
    collections: changes.filter(c => c.kind === "updateCollections").length,
    backups: props.plan?.backups.length ?? 0
  };
});

const destructiveArtworkCount = computed(
  () => props.plan?.changes.filter(c => c.kind === "writeArtwork" && c.destructive).length ?? 0
);

function changeCount(game: GameReview) {
  return Number(Boolean(game.shortcut)) + game.collections.length + game.artwork.length;
}

function collectionName(change: PlannedChange) {
  const match = change.title.match(/to (.+) collection$/);
  return match?.[1] ?? "Managed";
}

function artworkKind(change: PlannedChange) {
  const match = change.title.match(/^Set (\w+) artwork/);
  return titleCase(match?.[1] ?? "Artwork");
}

function artworkSource(change: PlannedChange) {
  const match = change.details.match(/from (\w+)/);
  return sourceLabel(match?.[1] ?? "");
}

function sourceLabel(source: string) {
  switch (source.toLowerCase()) {
    case "officialsteam": return "Official Steam";
    case "steamgriddb": return "SteamGridDB";
    case "localfile": return "Local file";
    case "existingcustom": return "Existing";
    default: return source || "Unknown";
  }
}

function titleCase(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
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
  <section v-if="applyResult" class="grid gap-4">
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

  <!-- Review state -->
  <section v-else class="grid gap-3">
    <div>
      <h1 class="text-[26px] font-bold leading-tight">Review</h1>
      <p class="text-secondary">Confirm the shortcuts, artwork, and collections that will be updated.</p>
    </div>

    <div v-if="!plan" class="grid min-h-55 place-items-center rounded-lg border border-border bg-surface-3 p-6 text-secondary">
      Preparing preview...
    </div>

    <template v-else>
      <!-- Summary stats -->
      <div class="flex flex-wrap gap-2">
        <div class="flex items-center gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
          <Gamepad2 :size="15" class="text-accent" />
          <strong>{{ summary.games }}</strong>
          <span class="text-xs text-secondary">games</span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
          <ListChecks :size="15" class="text-accent" />
          <strong>{{ summary.shortcuts }}</strong>
          <span class="text-xs text-secondary">shortcuts</span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
          <Image :size="15" class="text-accent" />
          <strong>{{ summary.artwork }}</strong>
          <span class="text-xs text-secondary">artwork</span>
        </div>
        <div v-if="summary.collections" class="flex items-center gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
          <Library :size="15" class="text-accent" />
          <strong>{{ summary.collections }}</strong>
          <span class="text-xs text-secondary">collections</span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
          <Save :size="15" class="text-accent" />
          <strong>{{ summary.backups }}</strong>
          <span class="text-xs text-secondary">backups</span>
        </div>
      </div>

      <!-- Notices -->
      <div class="flex min-h-10 items-center gap-2 rounded-md border border-border bg-surface-5 px-3 py-2 text-sm">
        <Check :size="16" class="shrink-0 text-accent" />
        <span>
          Backups will be created before any files are written.
          <span v-if="destructiveArtworkCount > 0" class="text-danger">
            {{ destructiveArtworkCount }} existing artwork file{{ destructiveArtworkCount === 1 ? "" : "s" }} will be replaced.
          </span>
        </span>
      </div>

      <div
        v-if="plan.warnings.length"
        class="flex min-h-10 items-center gap-2 rounded-md border border-warning-border bg-warning-bg px-3 py-2 text-sm text-warning"
      >
        <AlertTriangle :size="16" class="shrink-0" />
        <span>{{ plan.warnings.join(" ") }}</span>
      </div>

      <!-- Game list -->
      <div class="grid gap-2">
        <article
          v-for="game in games"
          :key="game.name"
          class="overflow-hidden rounded-lg border border-border bg-surface-3"
        >
          <div class="flex items-center gap-3 border-b border-border px-3 py-2.5">
            <div class="h-4 w-0.5 shrink-0 rounded-full bg-accent" />
            <strong class="min-w-0 flex-1 truncate">{{ game.name }}</strong>
            <span class="shrink-0 text-xs text-secondary">{{ changeCount(game) }} change{{ changeCount(game) === 1 ? "" : "s" }}</span>
          </div>
          <div class="flex flex-wrap gap-1.5 px-3 py-2.5">
            <span
              v-if="game.shortcut"
              class="inline-flex items-center gap-1.5 rounded-md border border-border bg-surface-5 px-2 py-1 text-xs"
            >
              <ListChecks :size="12" class="text-accent" />
              Shortcut
            </span>
            <span
              v-for="asset in game.artwork"
              :key="`${game.name}:${asset.kind}`"
              class="inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs"
              :class="asset.destructive ? 'border-danger-border-muted bg-danger-bg text-danger' : 'border-border bg-surface-5'"
            >
              <Image :size="12" :class="asset.destructive ? 'text-danger' : 'text-accent'" />
              <strong class="text-primary">{{ asset.kind }}</strong>
              <span class="text-secondary">{{ asset.source }}</span>
            </span>
            <span
              v-for="coll in game.collections"
              :key="`${game.name}:coll:${coll}`"
              class="inline-flex items-center gap-1.5 rounded-md border border-border bg-surface-5 px-2 py-1 text-xs text-secondary"
            >
              <Library :size="12" class="text-accent" />
              {{ coll }}
            </span>
          </div>
        </article>
      </div>

      <!-- Backup details -->
      <details class="group rounded-lg border border-border bg-surface-3">
        <summary class="flex cursor-pointer list-none items-center justify-between gap-3 px-3 py-2.5 text-sm">
          <span class="inline-flex items-center gap-2">
            <FolderArchive :size="15" />
            <strong>Backup and file details</strong>
            <span class="text-secondary">{{ plan.filesToChange.length }} files</span>
          </span>
          <ChevronDown :size="14" class="text-secondary transition-transform group-open:rotate-180" />
        </summary>
        <div class="grid gap-3 border-t border-border p-3 text-sm">
          <div>
            <h2 class="mb-1.5 font-semibold text-secondary">Backups</h2>
            <p v-if="plan.backups.length === 0" class="text-secondary">No existing Steam files need to be backed up.</p>
            <p v-for="backup in plan.backups" :key="backup.destination" class="path-cell">
              {{ backup.source }} → {{ backup.destination }}
            </p>
          </div>
          <div>
            <h2 class="mb-1.5 font-semibold text-secondary">Files to change</h2>
            <p v-for="file in plan.filesToChange" :key="file" class="path-cell">{{ file }}</p>
          </div>
        </div>
      </details>
    </template>
  </section>
</template>
