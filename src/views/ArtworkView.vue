<script setup lang="ts">
import { ImagePlus, RotateCcw } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ref } from "vue";
import UiButton from "../components/ui/UiButton.vue";
import type { ArtworkAsset, ArtworkKind, ImportCandidate } from "../types";

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

const slots: Array<{ kind: ArtworkKind; label: string; shape: string }> = [
  { kind: "header", label: "Header", shape: "aspect-[92/43]" },
  { kind: "capsule", label: "Capsule", shape: "aspect-[2/3]" },
  { kind: "hero", label: "Hero", shape: "aspect-[16/7]" },
  { kind: "logo", label: "Logo", shape: "aspect-video" },
  { kind: "icon", label: "Icon", shape: "aspect-square" }
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

    <div class="grid gap-3">
      <article
        v-for="candidate in candidates"
        :key="candidate.id"
        class="rounded-lg border border-fsa-line bg-fsa-panel-3 p-3"
      >
        <div class="mb-3 flex items-center justify-between gap-3">
          <div>
            <strong class="text-base">{{ candidate.name }}</strong>
            <p class="text-fsa-muted">
              <span v-if="candidate.matchedSteamAppId">Matched Steam AppID {{ candidate.matchedSteamAppId }}</span>
              <span v-else>No Steam match found yet</span>
            </p>
          </div>
          <span class="rounded-full border border-fsa-line px-2 py-1 text-xs text-fsa-muted">
            {{ selectedSlotCount(candidate) }} / {{ slots.length }} selected
          </span>
        </div>

        <div class="grid grid-cols-5 gap-2">
          <div
            v-for="slot in slots"
            :key="slot.kind"
            class="grid min-w-0 gap-2 rounded-md border border-fsa-line bg-[#1a1f25] p-2"
          >
            <div class="flex items-center justify-between gap-2">
              <strong>{{ slot.label }}</strong>
              <span class="truncate text-xs text-fsa-muted">
                {{ sourceLabel(selectedAsset(candidate, slot.kind) || existingAsset(candidate, slot.kind)) }}
              </span>
            </div>

            <div
              class="grid place-items-center overflow-hidden rounded-md border border-dashed border-[#596370] bg-fsa-panel-3"
              :class="slot.shape"
            >
              <img
                v-if="displayAsset(candidate, slot.kind)?.pathOrUrl && !previewErrored(displayAsset(candidate, slot.kind))"
                class="h-full w-full object-cover"
                :class="slot.kind === 'logo' || slot.kind === 'icon' ? 'object-contain p-2' : ''"
                :src="previewSrc(displayAsset(candidate, slot.kind))"
                @error="markPreviewErrored(displayAsset(candidate, slot.kind))"
                alt=""
              />
              <span v-else class="text-fsa-muted">Missing</span>
            </div>

            <div class="flex gap-2">
              <UiButton class="flex-1" variant="secondary" size="sm" @click="$emit('pick-artwork', candidate.id, slot.kind)">
                <ImagePlus :size="14" />
                Local
              </UiButton>
              <UiButton
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
