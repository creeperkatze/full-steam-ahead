<script setup lang="ts">
import { CheckCircle2, Clock, Loader2, Star } from '@lucide/vue'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { onMounted, ref } from 'vue'

import KofiIcon from '../assets/icons/kofi.svg?component'

defineProps<{
	error: string
}>()

defineSlots<{
	default?: () => unknown
	footer?: () => unknown
}>()

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
	<main class="flex min-h-0 flex-1 flex-col px-4 pb-2 text-primary">
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

		<div class="grid shrink-0 grid-cols-[1fr_auto_1fr] items-end gap-3 pt-2">
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
			<div class="flex min-h-9 items-center gap-2">
				<slot name="footer" />
			</div>
			<div class="flex items-center justify-end gap-3">
				<button
					type="button"
					class="flex shrink-0 cursor-pointer items-center gap-1.5 text-sm text-[#FF5E5B] transition-colors hover:text-[#ff8e8c]"
					@click="openUrl('https://ko-fi.com/creeperkatze')"
				>
					<KofiIcon class="size-4" aria-hidden="true" />
					<span>Donate</span>
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
