<script setup lang="ts">
defineProps<{
	as?: string
	active?: boolean
	interactive?: boolean
}>()
</script>

<template>
	<component
		:is="as || 'div'"
		class="flex items-center gap-3 rounded-lg border px-3 py-2.5 text-sm transition-colors"
		:class="[
			active ? 'border-accent/30 bg-accent-bg' : 'border-border/60 bg-surface-5',
			interactive && 'cursor-pointer hover:bg-surface-hover',
		]"
	>
		<slot name="leading" />
		<span class="min-w-0 flex-1 leading-snug">
			<slot />
		</span>
		<slot name="trailing" />
	</component>
</template>
