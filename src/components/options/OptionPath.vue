<script setup lang="ts">
import { FolderOpen, X } from '@lucide/vue'
import type { Component } from 'vue'

import UiButton from '../ui/Button.vue'

defineProps<{
	icon?: Component
	label: string
	description?: string
	modelValue: string
	placeholder?: string
}>()

defineEmits<{
	'update:modelValue': [value: string]
	browse: []
}>()
</script>

<template>
	<div class="flex min-w-0 flex-col gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2">
		<div class="flex min-w-0 items-center gap-3">
			<component :is="icon" v-if="icon" :size="18" class="shrink-0 text-secondary" />
			<div class="min-w-0 flex-1">
				<p class="text-sm font-medium">{{ label }}</p>
				<p v-if="description" class="mt-0.5 text-xs text-secondary">{{ description }}</p>
			</div>
		</div>
		<div class="flex items-center gap-2">
			<input
				:value="modelValue"
				:placeholder="placeholder"
				class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-4 px-2 text-sm text-primary"
				@input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
			/>
			<UiButton
				v-if="modelValue"
				size="icon"
				variant="ghost"
				title="Clear"
				@click="$emit('update:modelValue', '')"
			>
				<X :size="16" />
			</UiButton>
			<UiButton size="icon" variant="ghost" title="Browse" @click="$emit('browse')">
				<FolderOpen :size="16" />
			</UiButton>
		</div>
	</div>
</template>
