<script setup lang="ts">
import { Check, ChevronDown, FolderArchive, Gamepad2, Image, Library, ListChecks, Loader2, Save } from "@lucide/vue";
import { computed } from "vue";
import GameIcon from "../../../components/GameIcon.vue";
import ItemRow from "../../../components/ui/ItemRow.vue";
import { useAppState } from "../../../composables/useAppState";
import type { ApplyProgressEvent, ApplyResult, PlannedChange, PreviewPlan } from "../../../types";

const state = useAppState();

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

interface CollectionChange {
  name: string;
  destructive: boolean;
}

interface GameReview {
  name: string;
  shortcut?: PlannedChange;
  collections: CollectionChange[];
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
      game.collections.push({ name: collectionName(change), destructive: change.destructive });
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

const candidateByName = computed(() =>
  new Map(state.candidates.value.map(c => [c.name, c]))
);

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

function fileName(path: string) {
  const lastSep = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return lastSep >= 0 ? path.slice(lastSep + 1) : path;
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

      <!-- Game list -->
      <div class="grid gap-2">
        <article
          v-for="game in games"
          :key="game.name"
          class="overflow-hidden rounded-xl border border-border"
        >
          <div class="flex items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5">
            <GameIcon v-if="candidateByName.get(game.name)" :candidate="candidateByName.get(game.name)!" :size="20" />
            <strong class="min-w-0 flex-1 truncate text-base">{{ game.name }}</strong>
            <span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
              {{ changeCount(game) }} change{{ changeCount(game) === 1 ? "" : "s" }}
            </span>
          </div>

          <div class="grid gap-1.5 bg-surface-3 p-2">
            <ItemRow v-if="game.shortcut">
              <template #leading>
                <ListChecks :size="15" class="text-accent" />
              </template>
              Steam entry
              <template #trailing>
                <span v-if="game.shortcut.kind === 'addShortcut'" class="shrink-0 text-xs text-accent">New</span>
                <span v-else class="shrink-0 text-xs text-secondary">Update</span>
              </template>
            </ItemRow>

            <ItemRow
              v-for="asset in game.artwork"
              :key="`${game.name}:${asset.kind}`"
            >
              <template #leading>
                <Image :size="15" class="text-accent" />
              </template>
              <strong>{{ asset.kind }}</strong>
              <span class="text-secondary"> · {{ asset.source }}</span>
              <template #trailing>
                <span v-if="asset.destructive" class="shrink-0 text-xs text-secondary">Update</span>
                <span v-else class="shrink-0 text-xs text-accent">New</span>
              </template>
            </ItemRow>

            <ItemRow
              v-for="coll in game.collections"
              :key="`${game.name}:coll:${coll.name}`"
            >
              <template #leading>
                <Library :size="15" class="text-accent" />
              </template>
              {{ coll.name }}
              <template #trailing>
                <span v-if="coll.destructive" class="shrink-0 text-xs text-secondary">Use existing</span>
                <span v-else class="shrink-0 text-xs text-accent">New</span>
              </template>
            </ItemRow>
          </div>
        </article>
      </div>

      <!-- Backup details -->
      <details class="group overflow-hidden rounded-xl border border-border">
        <summary class="flex cursor-pointer list-none items-center justify-between gap-3 border-b border-transparent bg-surface-4 px-3 py-2.5 text-sm group-open:border-border">
          <span class="inline-flex items-center gap-2">
            <FolderArchive :size="15" />
            <strong>Backups</strong>
          </span>
          <span class="flex items-center gap-2">
            <span class="rounded-md border border-border px-2 py-1 text-xs text-secondary">{{ plan.backups.length }} files</span>
            <ChevronDown :size="14" class="text-secondary transition-transform group-open:rotate-180" />
          </span>
        </summary>
        <div class="grid gap-1.5 bg-surface-3 p-2">
          <ItemRow v-for="backup in plan.backups" :key="backup.destination">
            <template #leading>
              <FolderArchive :size="14" class="shrink-0 text-accent" />
            </template>
            <span class="truncate">{{ fileName(backup.source) }}</span>
          </ItemRow>
        </div>
      </details>
    </template>
  </section>
</template>
