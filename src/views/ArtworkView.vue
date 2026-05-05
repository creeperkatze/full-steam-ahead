<script setup lang="ts">
import { ImagePlus, RotateCcw } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref } from "vue";
import UiButton from "../components/ui/Button.vue";
import type { ArtworkAsset, ArtworkKind, ImportCandidate } from "../types/steam";

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

const slots: Array<{ kind: ArtworkKind; label: string; shape: string; image: string }> = [
  {
    kind: "header",
    label: "Header",
    shape: "h-[88px] w-full",
    image: "object-contain"
  },
  {
    kind: "capsule",
    label: "Capsule",
    shape: "aspect-[2/3] h-[116px]",
    image: "object-contain"
  },
  {
    kind: "hero",
    label: "Hero",
    shape: "h-[88px] w-full",
    image: "object-contain"
  },
  {
    kind: "logo",
    label: "Logo",
    shape: "h-[96px] w-full",
    image: "object-contain p-2"
  },
  {
    kind: "icon",
    label: "Icon",
    shape: "h-14 w-14",
    image: "object-contain"
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
  <section class="rounded-lg border border-fsa-line bg-fsa-panel p-4">
    <div class="mb-3 flex items-center justify-between gap-4">
      <div>
        <h1 class="text-[26px] font-bold leading-tight">Artwork</h1>
        <p class="text-fsa-muted">
          Official Steam artwork is selected automatically when a Steam match is found. Each slot can be replaced locally.
        </p>
      </div>
      <label class="inline-flex h-9 items-center gap-2 rounded-md border border-fsa-line bg-fsa-panel-3 px-3">
        <input
          type="checkbox"
          :checked="replaceExistingArtwork"
          @change="$emit('update:replaceExistingArtwork', ($event.target as HTMLInputElement).checked)"
        />
        Replace existing custom art
      </label>
    </div>

    <div class="grid gap-2.5">
      <article
        v-for="candidate in candidates"
        :key="candidate.id"
        class="grid grid-cols-[180px_minmax(0,1fr)] gap-3 rounded-lg border border-fsa-line bg-fsa-panel-3 p-3"
      >
        <div class="flex min-w-0 flex-col justify-between gap-3 border-r border-fsa-line pr-3">
          <div class="min-w-0">
            <strong class="block truncate text-base">{{ candidate.name }}</strong>
          </div>
          <span class="shrink-0 rounded-full border border-fsa-line px-2 py-1 text-xs text-fsa-muted">
            {{ selectedSlotCount(candidate) }} / {{ slots.length }} selected
          </span>
        </div>

        <div class="grid min-w-0 grid-cols-[minmax(180px,1.15fr)_116px_minmax(180px,1.15fr)_minmax(160px,1fr)_96px] gap-2">
          <div
            v-for="slot in slots"
            :key="slot.kind"
            class="grid min-w-0 grid-rows-[auto_1fr_auto] gap-2 overflow-hidden rounded-md border border-fsa-line bg-[#1a1f25] p-2"
          >
            <div class="flex min-w-0 items-center justify-between gap-2">
              <strong class="shrink-0 text-xs">{{ slot.label }}</strong>
              <span class="min-w-0 truncate text-xs text-fsa-muted">
                {{ sourceLabel(selectedAsset(candidate, slot.kind) || existingAsset(candidate, slot.kind)) }}
              </span>
            </div>

            <div class="grid min-h-[76px] place-items-center">
              <div
                class="grid place-items-center overflow-hidden rounded-md border border-dashed border-[#596370] bg-fsa-panel-3"
                :class="slot.shape"
              >
                <img
                  v-if="displayAsset(candidate, slot.kind)?.pathOrUrl && !previewErrored(displayAsset(candidate, slot.kind))"
                  class="h-full w-full"
                  :class="slot.image"
                  :src="previewSrc(displayAsset(candidate, slot.kind))"
                  @error="markPreviewErrored(displayAsset(candidate, slot.kind))"
                  alt=""
                />
                <span v-else class="px-2 text-xs text-fsa-muted">Missing</span>
              </div>
            </div>

            <div class="grid grid-cols-[1fr_32px] gap-2">
              <UiButton
                class="h-8 min-w-0 px-2 text-xs"
                variant="secondary"
                size="sm"
                title="Pick local artwork"
                @click="$emit('pick-artwork', candidate.id, slot.kind)"
              >
                <ImagePlus :size="14" />
                <span v-if="slot.kind !== 'icon'">Local</span>
              </UiButton>
              <UiButton
                class="h-8 w-8"
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
