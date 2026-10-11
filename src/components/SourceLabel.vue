<script setup lang="ts">
import { importSourceName } from '../helpers/sourceNames'
import type { ImportSource } from '../types'
import SourceIcon from './SourceIcon.vue'

defineProps<{ source: ImportSource }>()
</script>

<template>
	<span class="flex shrink-0 items-center gap-1 text-xs whitespace-nowrap text-secondary">
		<SourceIcon v-if="typeof source === 'string'" :source="source" class="size-3.5 shrink-0" />
		{{ importSourceName(source) }}
	</span>
</template>
