<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'

import { displayAsset } from '../helpers/artwork'
import type { ImportCandidate } from '../types'

const props = defineProps<{ candidate: ImportCandidate; size?: number }>()

const errored = ref(false)

const iconSrc = computed(() => {
	const asset = displayAsset(props.candidate, 'icon')
	if (!asset?.pathOrUrl) return null
	return asset.source === 'localFile' || asset.source === 'existingCustom'
		? convertFileSrc(asset.pathOrUrl)
		: asset.pathOrUrl
})

watch(iconSrc, () => {
	errored.value = false
})

const px = computed(() => `${props.size ?? 20}px`)
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
