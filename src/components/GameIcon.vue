<script setup lang="ts">
import { computed, ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ImportCandidate } from "../types";

const props = defineProps<{ candidate: ImportCandidate; size?: number }>();

const errored = ref(false);

const iconSrc = computed(() => {
  errored.value = false;
  const asset =
    props.candidate.artwork.proposed.find(a => a.kind === "icon") ??
    props.candidate.artwork.existing.find(a => a.kind === "icon");
  if (!asset) return null;
  return asset.source === "localFile" || asset.source === "existingCustom"
    ? convertFileSrc(asset.pathOrUrl)
    : asset.pathOrUrl;
});

const px = computed(() => `${props.size ?? 20}px`);
</script>

<template>
  <img
    v-if="iconSrc && !errored"
    :src="iconSrc"
    class="shrink-0 rounded-sm object-contain"
    :style="{ width: px, height: px }"
    alt=""
    @error="errored = true"
  />
</template>
