<script setup lang="ts">
import { CheckCircle2, FolderOpen, X, XCircle } from '@lucide/vue'
import type { Component } from 'vue'

import UiButton from '../ui/Button.vue'

defineProps<{
	icon?: Component
	label: string
	description?: string
	modelValue: string
	placeholder?: string
	valid?: boolean | null
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
			<div class="relative min-w-0 flex-1">
				<input
					:value="modelValue"
					:placeholder="placeholder"
					class="h-9 w-full rounded-md border bg-surface-4 px-2 text-sm text-primary"
					:class="[
						valid === false ? 'border-danger-border' : 'border-border',
						valid === null || valid === undefined ? '' : 'pr-7',
					]"
					@input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
				/>
				<CheckCircle2
					v-if="valid === true"
					:size="15"
					class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 text-accent"
				/>
				<XCircle
					v-else-if="valid === false"
					:size="15"
					class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 text-danger"
				/>
			</div>
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
