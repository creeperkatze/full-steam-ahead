<script setup lang="ts">
import { Settings, X } from '@lucide/vue'

import Logo from '../assets/logo.svg?component'
import UiButton from './ui/Button.vue'

defineProps<{
	activeStep: number
	error: string
	settingsOpen: boolean
}>()

defineEmits<{
	'select-step': [index: number]
	'toggle-settings': []
}>()

const steps = ['Start', 'Sources', 'Artwork', 'Review', 'Done']
</script>

<template>
	<main class="flex h-screen flex-col gap-3 bg-surface-2 p-3 text-primary">
		<header class="grid h-15 shrink-0 grid-cols-[280px_1fr_auto] items-center gap-5">
			<div class="flex items-center">
				<Logo class="h-9 w-auto" aria-label="Full Steam Ahead" />
			</div>

			<nav v-if="!settingsOpen" class="grid grid-cols-5 gap-2" aria-label="Import progress">
				<button
					v-for="(step, index) in steps"
					:key="step"
					type="button"
					class="flex min-h-9 items-center gap-2 rounded-md border px-3 text-left text-secondary transition-colors hover:border-accent hover:bg-accent-bg hover:text-primary"
					:class="
						activeStep >= index
							? 'border-accent bg-accent-bg text-primary'
							: 'border-border bg-surface-5'
					"
					@click="$emit('select-step', index)"
				>
					<b
						class="grid size-5 place-items-center rounded-full text-xs"
						:class="
							activeStep >= index
								? 'bg-accent text-accent-contrast'
								: 'bg-border-muted text-secondary'
						"
					>
						{{ index + 1 }}
					</b>
					{{ step }}
				</button>
			</nav>
			<div v-else aria-hidden="true" />

			<div class="flex items-center gap-2">
				<UiButton
					size="icon"
					variant="ghost"
					:title="settingsOpen ? 'Close settings' : 'Settings'"
					:active="settingsOpen"
					@click="$emit('toggle-settings')"
				>
					<X v-if="settingsOpen" :size="18" />
					<Settings v-else :size="17" />
				</UiButton>
			</div>
		</header>

		<div
			class="flex min-h-0 flex-1 flex-col overflow-y-auto rounded-xl border border-border bg-surface-1 px-5 py-4"
		>
			<div
				v-if="error"
				class="mb-3 rounded-md border border-danger-border bg-danger-bg px-3 py-2 text-danger"
			>
				{{ error }}
			</div>
			<slot />
		</div>

		<slot name="footer" />
	</main>
</template>
