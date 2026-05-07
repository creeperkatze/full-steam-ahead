<script setup lang="ts">
import { ImagePlus, RotateCcw } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref } from "vue";
import UiButton from "../../../components/ui/Button.vue";
import type { ArtworkAsset, ArtworkKind, ImportCandidate } from "../../../types/steam";

const props = defineProps<{
  candidates: ImportCandidate[];
  customArtwork: Record<string, string>;
  replaceExistingArtwork: boolean;
}>();

defineEmits<{
  "pick-artwork": [candidateId: string, kind: ArtworkKind];
  "use-official-artwork": [candidateId: string, kind: ArtworkKind];
  "update:replaceExistingArtwork": [value: boolean];
}>();

const slots: Array<{ kind: ArtworkKind; label: string; preview: string }> = [
  {
    kind: "header",
    label: "Header",
    preview: "h-32 p-2"
  },
  {
    kind: "capsule",
    label: "Capsule",
    preview: "h-44 p-2"
  },
  {
    kind: "hero",
    label: "Hero",
    preview: "h-32 p-2"
  },
  {
    kind: "logo",
    label: "Logo",
    preview: "h-32 p-4"
  },
  {
    kind: "icon",
    label: "Icon",
    preview: "h-32 p-5"
  }
];

const brokenPreviewUrls = ref<Record<string, true>>({});

function artworkKey(candidateId: string, kind: ArtworkKind) {
  return `${candidateId}:${kind}`;
}

function selectedAsset(candidate: ImportCandidate, kind: ArtworkKind): ArtworkAsset | undefined {
  const localPath = props.customArtwork[artworkKey(candidate.id, kind)];
  if (localPath) {
    return {
      kind,
      pathOrUrl: localPath,
      source: "localFile",
      willReplaceExisting: true
    };
  }
  return candidate.artwork.proposed.find((asset) => asset.kind === kind);
}

function existingAsset(candidate: ImportCandidate, kind: ArtworkKind): ArtworkAsset | undefined {
  return candidate.artwork.existing.find((asset) => asset.kind === kind);
}

function sourceLabel(asset?: ArtworkAsset) {
  if (!asset) return "Missing";
  if (asset.source === "officialSteam") return "Official Steam";
  if (asset.source === "localFile") return "Local file";
  if (asset.source === "existingCustom") return "Existing custom";
  return "SteamGridDB";
}

function previewSrc(asset?: ArtworkAsset) {
  if (!asset) return "";
  return asset.source === "localFile" || asset.source === "existingCustom"
    ? convertFileSrc(asset.pathOrUrl)
    : asset.pathOrUrl;
}

function previewErrored(asset?: ArtworkAsset) {
  const src = previewSrc(asset);
  return src ? brokenPreviewUrls.value[src] : false;
}

function markPreviewErrored(asset?: ArtworkAsset) {
  const src = previewSrc(asset);
  if (src) brokenPreviewUrls.value[src] = true;
}

function displayAsset(candidate: ImportCandidate, kind: ArtworkKind) {
  return selectedAsset(candidate, kind) || existingAsset(candidate, kind);
}

function selectedSlotCount(candidate: ImportCandidate) {
  return slots.filter((slot) => selectedAsset(candidate, slot.kind)).length;
}
</script>

<template>
  <section class="grid gap-3">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h1 class="text-[26px] font-bold leading-tight">Artwork</h1>
        <p class="text-secondary">
          Official Steam artwork is selected automatically when a Steam match is found. Each slot can be replaced locally.
        </p>
      </div>
      <label class="inline-flex h-9 items-center gap-2 rounded-md border border-border bg-surface-5 px-3">
        <input
          type="checkbox"
          :checked="replaceExistingArtwork"
          @change="$emit('update:replaceExistingArtwork', ($event.target as HTMLInputElement).checked)"
        />
        Replace existing custom art
      </label>
    </div>

    <div
      v-if="candidates.length === 0"
      class="grid min-h-[220px] place-items-center rounded-lg border border-dashed border-border-dashed bg-surface-3 p-6 text-secondary"
    >
      Select games before reviewing artwork.
    </div>

    <div v-else class="grid gap-3">
      <article
        v-for="candidate in candidates"
        :key="candidate.id"
        class="overflow-hidden rounded-lg border border-border bg-surface-3"
      >
        <header class="flex min-h-14 items-center justify-between gap-3 border-b border-border bg-surface-4 px-3 py-2.5">
          <div class="min-w-0">
            <strong class="block truncate text-base">{{ candidate.name }}</strong>
            <span class="text-xs text-secondary">Artwork slots</span>
          </div>
          <span class="shrink-0 rounded-full border border-border px-2 py-1 text-xs text-secondary">
            {{ selectedSlotCount(candidate) }} / {{ slots.length }} selected
          </span>
        </header>

        <div class="grid min-w-0 grid-cols-[repeat(auto-fit,minmax(190px,1fr))] gap-3 p-3">
          <div
            v-for="slot in slots"
            :key="slot.kind"
            class="grid min-w-0 grid-rows-[auto_auto_auto] gap-2 rounded-md border border-border bg-surface-5 p-2.5"
          >
            <div class="flex min-w-0 items-center justify-between gap-2">
              <strong class="shrink-0 text-xs">{{ slot.label }}</strong>
              <span class="min-w-0 truncate text-xs text-secondary">
                {{ sourceLabel(selectedAsset(candidate, slot.kind) || existingAsset(candidate, slot.kind)) }}
              </span>
            </div>

            <div
              class="flex w-full items-center justify-center rounded-md border border-dashed border-border-dashed bg-surface-inset"
              :class="slot.preview"
            >
              <img
                v-if="displayAsset(candidate, slot.kind)?.pathOrUrl && !previewErrored(displayAsset(candidate, slot.kind))"
                class="max-h-full max-w-full object-contain"
                :src="previewSrc(displayAsset(candidate, slot.kind))"
                @error="markPreviewErrored(displayAsset(candidate, slot.kind))"
                alt=""
              />
              <span v-else class="px-2 text-xs text-secondary">Missing</span>
            </div>

            <div class="grid grid-cols-[1fr_36px] gap-2">
              <UiButton
                class="h-9 min-w-0 px-2 text-xs"
                variant="secondary"
                size="sm"
                title="Pick local artwork"
                @click="$emit('pick-artwork', candidate.id, slot.kind)"
              >
                <span v-if="slot.kind !== 'icon'">Local</span>
                <template #icon>
                  <ImagePlus :size="14" />
                </template>
              </UiButton>
              <UiButton
                class="h-9 w-9"
                size="icon"
                variant="ghost"
                title="Use official Steam artwork"
                :disabled="!candidate.artwork.proposed.some((asset) => asset.kind === slot.kind && asset.source === 'officialSteam')"
                @click="$emit('use-official-artwork', candidate.id, slot.kind)"
              >
                <RotateCcw :size="14" />
              </UiButton>
            </div>
          </div>
        </div>
      </article>
    </div>
  </section>
</template>
