<script setup lang="ts">
import { Maximize2, Minimize2, Minus, Settings, X } from '@lucide/vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { onMounted, onUnmounted, ref } from 'vue'

import Logo from '../assets/logo.svg?component'
import UiButton from './ui/Button.vue'

defineProps<{
	activeStep: number
	settingsOpen: boolean
}>()

defineEmits<{
	'select-step': [index: number]
	'toggle-settings': []
}>()

const steps = ['Start', 'Sources', 'Artwork', 'Review', 'Done']

const win = getCurrentWindow()
const isMaximized = ref(false)

async function updateMaximized() {
	isMaximized.value = await win.isMaximized()
}

let unlisten: (() => void) | undefined

onMounted(async () => {
	await updateMaximized()
	unlisten = await win.onResized(updateMaximized)
})

onUnmounted(() => unlisten?.())
</script>

<template>
	<header
		class="grid h-18 shrink-0 select-none grid-cols-[auto_1fr_auto] items-center border-b border-border"
		data-tauri-drag-region
	>
		<div class="flex items-center px-3">
			<button
				type="button"
				class="cursor-pointer rounded opacity-90 transition-opacity hover:opacity-100"
				title="View on GitHub"
				@click="openUrl('https://github.com/creeperkatze/full-steam-ahead')"
			>
				<Logo class="h-9 w-auto" aria-label="Full Steam Ahead" />
			</button>
		</div>

		<nav v-if="!settingsOpen" class="grid grid-cols-5 gap-2 px-2" aria-label="Import progress">
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

		<div class="flex items-stretch self-stretch">
			<div class="flex items-center px-2">
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
			<button
				type="button"
				title="Minimize"
				class="flex w-11 items-center justify-center text-secondary transition-colors hover:bg-surface-hover hover:text-primary"
				@click="win.minimize()"
			>
				<Minus :size="14" />
			</button>
			<button
				type="button"
				:title="isMaximized ? 'Restore' : 'Maximize'"
				class="flex w-11 items-center justify-center text-secondary transition-colors hover:bg-surface-hover hover:text-primary"
				@click="win.toggleMaximize()"
			>
				<Minimize2 v-if="isMaximized" :size="13" />
				<Maximize2 v-else :size="13" />
			</button>
			<button
				type="button"
				title="Close"
				class="flex w-11 items-center justify-center text-secondary transition-colors hover:bg-red-700 hover:text-white"
				@click="win.close()"
			>
				<X :size="15" />
			</button>
		</div>
	</header>
</template>
