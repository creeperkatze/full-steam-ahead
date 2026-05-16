<script setup lang="ts">
import { CheckCircle2, Clock, Loader2, Settings, Star, X } from '@lucide/vue'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { onMounted, ref } from 'vue'

import KofiIcon from '../assets/icons/kofi.svg?component'
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

const version = ref('')
const updateChecking = ref(true)
const isLatest = ref(false)
const latestVersion = ref<string | null>(null)

async function checkForUpdates() {
	try {
		const CACHE_KEY = 'updateCheckCache'
		const CACHE_TTL = 60 * 60 * 1000
		const raw = window.localStorage.getItem(CACHE_KEY)
		const cached = raw ? (JSON.parse(raw) as { tag: string; ts: number }) : null
		let tag: string
		if (cached && Date.now() - cached.ts < CACHE_TTL) {
			tag = cached.tag
		} else {
			const res = await fetch(
				'https://api.github.com/repos/creeperkatze/full-steam-ahead/releases/latest',
			)
			if (!res.ok) throw new Error(`HTTP ${res.status}`)
			const data = (await res.json()) as { tag_name?: string }
			tag = data.tag_name?.replace(/^v/, '') ?? ''
			window.localStorage.setItem(CACHE_KEY, JSON.stringify({ tag, ts: Date.now() }))
		}
		if (tag && tag !== version.value) latestVersion.value = tag
		else if (tag) isLatest.value = true
	} catch {
		// silently ignore
	} finally {
		updateChecking.value = false
	}
}

onMounted(async () => {
	version.value = await getVersion().catch(() => '')
	await checkForUpdates()
})
</script>

<template>
	<main class="flex h-screen flex-col gap-3 bg-surface-2 p-3 text-primary">
		<header class="grid h-15 shrink-0 grid-cols-[280px_1fr_auto] items-center gap-5">
			<div class="flex items-center">
				<button
					type="button"
					class="cursor-pointer rounded opacity-90 transition-opacity hover:opacity-100"
					title="View on GitHub"
					@click="openUrl('https://github.com/creeperkatze/full-steam-ahead')"
				>
					<Logo class="h-9 w-auto" aria-label="Full Steam Ahead" />
				</button>
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

		<div class="grid shrink-0 grid-cols-[1fr_auto_1fr] items-end gap-3">
			<div class="flex min-w-0 items-center gap-2">
				<span class="shrink-0 text-sm text-secondary">v{{ version }}</span>
				<span v-if="updateChecking" class="flex min-w-0 items-center gap-1 text-sm text-secondary">
					<Loader2 class="size-3.5 shrink-0 animate-spin" aria-hidden="true" />
					<span class="truncate">Checking for updates</span>
				</span>
				<button
					v-else-if="isLatest"
					type="button"
					class="flex min-w-0 cursor-pointer items-center gap-1 text-sm text-green-500 transition-colors hover:text-green-400"
					@click="openUrl('https://github.com/creeperkatze/full-steam-ahead/releases/latest')"
				>
					<CheckCircle2 class="size-3.5 shrink-0" aria-hidden="true" />
					<span class="truncate">Latest version</span>
				</button>
				<button
					v-else-if="latestVersion"
					type="button"
					class="flex min-w-0 cursor-pointer items-center gap-1 text-sm text-yellow-500 transition-colors hover:text-yellow-400"
					@click="openUrl('https://github.com/creeperkatze/full-steam-ahead/releases/latest')"
				>
					<Clock class="size-3.5 shrink-0" aria-hidden="true" />
					<span class="truncate">Update available</span>
				</button>
			</div>
			<div class="flex items-center gap-2">
				<slot name="footer" />
			</div>
			<div class="flex items-center justify-end gap-3">
				<button
					type="button"
					class="flex shrink-0 cursor-pointer items-center gap-1.5 text-sm text-[#FF5E5B] transition-colors hover:text-[#ff8e8c]"
					@click="openUrl('https://ko-fi.com/creeperkatze')"
				>
					<KofiIcon class="size-4" aria-hidden="true" />
					<span>Support</span>
				</button>
				<button
					type="button"
					class="flex shrink-0 cursor-pointer items-center gap-1.5 text-sm text-yellow-500 transition-colors hover:text-yellow-300"
					@click="openUrl('https://github.com/creeperkatze/full-steam-ahead')"
				>
					<Star class="size-4 shrink-0" aria-hidden="true" />
					<span>On GitHub</span>
				</button>
			</div>
		</div>
	</main>
</template>
