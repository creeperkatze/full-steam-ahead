<script setup lang="ts">
import { FolderOpen, X } from '@lucide/vue'

import UiButton from '../ui/Button.vue'
import UiToggle from '../ui/Toggle.vue'

defineProps<{
	label: string
	enabled: boolean
	customPath: string
	pathSupported?: boolean
}>()

defineEmits<{
	'update:enabled': [value: boolean]
	'update:customPath': [value: string]
	browse: []
}>()
</script>

<template>
	<div class="flex min-w-0 flex-col gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
		<div class="flex min-w-0 items-center gap-3">
			<p class="min-w-0 flex-1 truncate text-sm font-medium">{{ label }}</p>
			<UiToggle :model-value="enabled" @update:model-value="$emit('update:enabled', $event)" />
		</div>
		<div v-if="pathSupported !== false" class="flex items-center gap-2">
			<input
				:value="customPath"
				placeholder="Auto-detected"
				class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-4 px-2 text-sm text-primary"
				@input="$emit('update:customPath', ($event.target as HTMLInputElement).value)"
			/>
			<UiButton
				v-if="customPath"
				size="icon"
				variant="ghost"
				title="Clear"
				@click="$emit('update:customPath', '')"
			>
				<X :size="16" />
			</UiButton>
			<UiButton size="icon" variant="ghost" title="Browse" @click="$emit('browse')">
				<FolderOpen :size="16" />
			</UiButton>
		</div>
	</div>
</template>
