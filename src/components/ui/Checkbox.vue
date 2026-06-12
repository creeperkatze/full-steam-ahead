<script setup lang="ts">
import { Check } from '@lucide/vue'

defineProps<{
	modelValue: boolean
	disabled?: boolean
}>()

defineEmits<{
	'update:modelValue': [value: boolean]
}>()
</script>

<template>
	<span class="relative inline-flex size-4 shrink-0">
		<input
			type="checkbox"
			:checked="modelValue"
			:disabled="disabled"
			class="absolute inset-0 cursor-pointer opacity-0 disabled:cursor-not-allowed"
			@change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
		/>
		<span
			class="pointer-events-none inline-flex size-4 items-center justify-center rounded border transition-colors"
			:class="[
				modelValue ? 'border-accent bg-accent-strong' : 'border-border bg-surface-5',
				disabled ? 'opacity-60' : '',
			]"
		>
			<Check v-if="modelValue" class="size-3 text-white" :stroke-width="3" />
		</span>
	</span>
</template>
