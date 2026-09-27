<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		variant?: 'default' | 'primary' | 'danger'
		size?: 'sm' | 'md' | 'icon'
		type?: 'button' | 'submit' | 'reset'
		disabled?: boolean
		active?: boolean
	}>(),
	{
		variant: 'default',
		size: 'md',
		type: 'button',
		disabled: false,
		active: false,
	},
)

const classes = computed(() => [
	'inline-flex items-center justify-center gap-2 rounded-md border transition-colors disabled:cursor-not-allowed disabled:opacity-45',
	props.size === 'icon' ? 'h-9 w-9 p-0' : props.size === 'sm' ? 'h-8 px-3' : 'h-9 px-3',
	props.variant === 'primary' &&
		'border-accent bg-accent-strong font-bold text-accent-contrast hover:bg-accent-hover',
	props.variant === 'default' && 'border-border bg-surface-5 text-primary hover:bg-surface-4',
	props.variant === 'danger' &&
		'border-danger-border bg-danger-control text-danger hover:bg-danger-hover',
	props.active && 'border-accent bg-accent-bg text-primary',
])

defineSlots<{
	default?: () => unknown
	icon?: () => unknown
}>()
</script>

<template>
	<button :type="type" :disabled="disabled" :class="classes">
		<slot />
		<slot name="icon" />
	</button>
</template>
