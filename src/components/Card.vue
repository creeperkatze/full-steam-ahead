<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Component } from 'vue'

withDefaults(
	defineProps<{
		href: string
		icon: Component
		color: string
		title: string
		description?: string
		highlight?: boolean
	}>(),
	{
		description: undefined,
		highlight: false,
	},
)
</script>

<template>
	<button
		type="button"
		class="group flex w-full cursor-pointer items-center gap-2.5 rounded-lg border px-2.5 py-2 text-left transition-colors"
		:class="highlight ? 'card-highlight' : 'border-border bg-surface-3 hover:bg-surface-4'"
		:style="highlight ? { '--c': color } : undefined"
		@click="openUrl(href)"
	>
		<component
			:is="icon"
			class="size-5 shrink-0 opacity-75 transition-opacity group-hover:opacity-100"
			:style="{ color }"
		/>
		<div class="min-w-0 flex-1">
			<div class="text-sm leading-tight font-semibold text-primary">{{ title }}</div>
			<div v-if="description" class="mt-px text-xs text-secondary">{{ description }}</div>
		</div>
	</button>
</template>

<style>
.card-highlight {
	background-color: color-mix(in srgb, var(--c) 8%, var(--color-surface-3));
	border-color: color-mix(in srgb, var(--c) 22%, transparent);
}

.card-highlight:hover {
	background-color: color-mix(in srgb, var(--c) 22%, var(--color-surface-3));
	border-color: color-mix(in srgb, var(--c) 60%, transparent);
}
</style>
