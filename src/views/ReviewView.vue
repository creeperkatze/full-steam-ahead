<script setup lang="ts">
import { AlertTriangle, Check, ChevronDown, FolderArchive, Image, Library, ListChecks } from "@lucide/vue";
import { computed } from "vue";
import type { ApplyResult, PlannedChange, PreviewPlan } from "../types/steam";

const props = defineProps<{
  plan: PreviewPlan | null;
  applyResult: ApplyResult | null;
}>();

interface GameReview {
  name: string;
  shortcut?: PlannedChange;
  collections: string[];
  artwork: Array<{
    kind: string;
    source: string;
    destructive: boolean;
  }>;
}

const games = computed(() => {
  const grouped = new Map<string, GameReview>();

  for (const change of props.plan?.changes ?? []) {
    const name = gameName(change);
    if (!name) continue;

    const game = grouped.get(name) ?? {
      name,
      collections: [],
      artwork: []
    };

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

  return Array.from(grouped.values()).sort((left, right) => left.name.localeCompare(right.name));
});

const summary = computed(() => {
  const changes = props.plan?.changes ?? [];
  return {
    games: games.value.length,
    shortcuts: changes.filter((change) => change.kind === "addShortcut" || change.kind === "updateShortcut").length,
    artwork: changes.filter((change) => change.kind === "writeArtwork").length,
    collections: changes.filter((change) => change.kind === "updateCollections").length,
    backups: props.plan?.backups.length ?? 0
  };
});

const destructiveArtworkCount = computed(
  () => props.plan?.changes.filter((change) => change.kind === "writeArtwork" && change.destructive).length ?? 0
);

function gameName(change: PlannedChange) {
  if (change.kind === "addShortcut" || change.kind === "updateShortcut") {
    return change.title.replace(/^Add shortcut for /, "").replace(/^Update shortcut for /, "");
  }

  if (change.kind === "updateCollections") {
    const match = change.title.match(/^Add (.+) to (.+) collection$/);
    return match?.[1] ?? "";
  }

  const match = change.title.match(/^Set \w+ artwork for (.+)$/);
  return match?.[1] ?? "";
}

function collectionName(change: PlannedChange) {
  const match = change.title.match(/^Add .+ to (.+) collection$/);
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
  if (source === "officialSteam") return "Official Steam";
  if (source === "steamGridDb") return "SteamGridDB";
  if (source === "localFile") return "Local file";
  if (source === "existingCustom") return "Existing custom";
  return source || "Selected source";
}

function titleCase(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
</script>

<template>
  <section class="grid gap-3">
    <div>
      <h1 class="text-[26px] font-bold leading-tight">Final Review</h1>
      <p class="text-secondary">Confirm the Steam shortcuts, artwork, and collections that will be updated.</p>
    </div>

    <div v-if="!plan" class="grid min-h-[220px] place-items-center rounded-lg border border-border bg-surface-3 p-6 text-secondary">
      Preparing preview...
    </div>

    <template v-else>
      <div class="grid grid-cols-[repeat(5,minmax(0,1fr))] gap-2">
        <div class="rounded-lg border border-border bg-surface-3 p-3">
          <span class="text-xs uppercase text-secondary">Games</span>
          <strong class="block text-xl">{{ summary.games }}</strong>
        </div>
        <div class="rounded-lg border border-border bg-surface-3 p-3">
          <span class="text-xs uppercase text-secondary">Shortcuts</span>
          <strong class="block text-xl">{{ summary.shortcuts }}</strong>
        </div>
        <div class="rounded-lg border border-border bg-surface-3 p-3">
          <span class="text-xs uppercase text-secondary">Artwork</span>
          <strong class="block text-xl">{{ summary.artwork }}</strong>
        </div>
        <div class="rounded-lg border border-border bg-surface-3 p-3">
          <span class="text-xs uppercase text-secondary">Collections</span>
          <strong class="block text-xl">{{ summary.collections }}</strong>
        </div>
        <div class="rounded-lg border border-border bg-surface-3 p-3">
          <span class="text-xs uppercase text-secondary">Backups</span>
          <strong class="block text-xl">{{ summary.backups }}</strong>
        </div>
      </div>

      <div class="flex min-h-10 items-center gap-2 rounded-md border border-border bg-surface-5 px-3 py-2">
        <Check :size="18" />
        <span>
          Backups will be created before writing Steam files.
          <span v-if="destructiveArtworkCount > 0" class="text-danger">
            {{ destructiveArtworkCount }} existing artwork file{{ destructiveArtworkCount === 1 ? "" : "s" }} will be replaced.
          </span>
        </span>
      </div>

      <div
        v-if="plan.warnings.length"
        class="flex min-h-10 items-center gap-2 rounded-md border border-warning-border bg-warning-bg px-3 py-2 text-warning"
      >
        <AlertTriangle :size="18" />
        <span>{{ plan.warnings.join(" ") }}</span>
      </div>

      <div class="grid gap-3">
        <article
          v-for="game in games"
          :key="game.name"
          class="overflow-hidden rounded-lg border border-border bg-surface-3"
        >
          <header class="flex min-h-14 items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2.5">
            <strong class="min-w-0 truncate text-base">{{ game.name }}</strong>
            <span class="shrink-0 rounded-full border border-border px-2 py-1 text-xs text-secondary">
              {{ Number(Boolean(game.shortcut)) + game.collections.length + game.artwork.length }} updates
            </span>
          </header>

          <div class="grid gap-3 p-3">
            <div v-if="game.shortcut" class="grid grid-cols-[20px_1fr] gap-2">
              <ListChecks :size="17" class="mt-0.5 text-accent" />
              <div>
                <strong class="block">Steam shortcut</strong>
                <p class="text-secondary">{{ game.shortcut.details }}</p>
              </div>
            </div>

            <div v-if="game.artwork.length" class="grid grid-cols-[20px_1fr] gap-2">
              <Image :size="17" class="mt-0.5 text-accent" />
              <div>
                <strong class="block">Artwork</strong>
                <div class="mt-2 flex flex-wrap gap-2">
                  <span
                    v-for="asset in game.artwork"
                    :key="`${game.name}:${asset.kind}`"
                    class="inline-flex items-center gap-1 rounded-full border px-2 py-1 text-xs"
                    :class="asset.destructive ? 'border-danger-border-muted text-danger' : 'border-border text-secondary'"
                  >
                    <strong class="text-primary">{{ asset.kind }}</strong>
                    {{ asset.source }}
                  </span>
                </div>
              </div>
            </div>

            <div v-if="game.collections.length" class="grid grid-cols-[20px_1fr] gap-2">
              <Library :size="17" class="mt-0.5 text-accent" />
              <div>
                <strong class="block">Collections</strong>
                <p class="text-secondary">
                  Add to {{ game.collections.join(", ") }}.
                  Only app-managed collections will be changed.
                </p>
              </div>
            </div>
          </div>
        </article>
      </div>

      <details class="group rounded-lg border border-border bg-surface-3">
        <summary class="flex cursor-pointer list-none items-center justify-between gap-3 px-3 py-2.5">
          <span class="inline-flex items-center gap-2">
            <FolderArchive :size="17" />
            <strong>Backup and file details</strong>
            <span class="text-secondary">{{ plan.filesToChange.length }} files checked</span>
          </span>
          <ChevronDown :size="16" class="transition-transform group-open:rotate-180" />
        </summary>
        <div class="grid gap-3 border-t border-border p-3">
          <div>
            <h2 class="mb-2 text-sm font-semibold">Backups</h2>
            <p v-if="plan.backups.length === 0" class="text-secondary">No existing Steam files need to be backed up.</p>
            <p v-for="backup in plan.backups" :key="backup.destination" class="path-cell">
              {{ backup.source }} -> {{ backup.destination }}
            </p>
          </div>
          <div>
            <h2 class="mb-2 text-sm font-semibold">Files to change</h2>
            <p v-for="file in plan.filesToChange" :key="file" class="path-cell">{{ file }}</p>
          </div>
        </div>
      </details>

      <div v-if="applyResult" class="rounded-lg border border-border bg-surface-3 p-3">
        <h2 class="mb-2 text-base font-semibold">Applied</h2>
        <p>{{ applyResult.appliedChanges.length }} changes applied.</p>
        <p>{{ applyResult.backupsCreated.length }} backups created.</p>
      </div>
    </template>
  </section>
</template>
