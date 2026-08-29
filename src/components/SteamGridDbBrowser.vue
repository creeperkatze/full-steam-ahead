<script setup lang="ts">
import { Loader2, Search, X } from '@lucide/vue'
import { ref, watch } from 'vue'

import { api } from '../helpers/api'
import type { ArtworkKind, SteamGridDbGame, SteamGridDbImage } from '../types'
import Modal from './Modal.vue'
import UiButton from './ui/Button.vue'

const props = defineProps<{
	apiKey: string
	kind: ArtworkKind
	initialQuery: string
	allowNsfw: boolean
}>()

const emit = defineEmits<{
	close: []
	select: [image: SteamGridDbImage]
}>()

const query = ref(props.initialQuery)
const games = ref<SteamGridDbGame[]>([])
const selectedGame = ref<SteamGridDbGame | null>(null)
const images = ref<SteamGridDbImage[]>([])
const searching = ref(false)
const loadingImages = ref(false)
const error = ref<string | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | undefined

function describeError(e: unknown, fallback: string): string {
	console.error(e)
	if (typeof e === 'string' && e) return e
	const message = (e as { message?: unknown } | null)?.message
	if (typeof message === 'string' && message) return message
	try {
		return JSON.stringify(e)
	} catch {
		return fallback
	}
}

async function runSearch() {
	const term = query.value.trim()
	if (!term) {
		games.value = []
		selectedGame.value = null
		images.value = []
		return
	}
	searching.value = true
	error.value = null
	try {
		games.value = await api.steamGridDbSearch(props.apiKey, term)
		if (games.value.length > 0) {
			await selectGame(games.value[0])
		} else {
			selectedGame.value = null
			images.value = []
		}
	} catch (e: unknown) {
		error.value = describeError(e, 'Search failed.')
	} finally {
		searching.value = false
	}
}

async function selectGame(game: SteamGridDbGame) {
	selectedGame.value = game
	loadingImages.value = true
	error.value = null
	try {
		images.value = await api.steamGridDbImages(props.apiKey, game.id, props.kind, props.allowNsfw)
	} catch (e: unknown) {
		error.value = describeError(e, 'Could not load images.')
		images.value = []
	} finally {
		loadingImages.value = false
	}
}

watch(query, () => {
	clearTimeout(searchTimer)
	searchTimer = setTimeout(runSearch, 400)
})

runSearch()

function pick(image: SteamGridDbImage) {
	emit('select', image)
}
</script>

<template>
	<Modal model-value fullscreen @update:model-value="emit('close')">
		<div class="flex items-center gap-2 border-b border-border px-4 py-3">
			<Search :size="16" class="shrink-0 text-secondary" />
			<input
				v-model="query"
				class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-4 px-2 text-sm text-primary"
				placeholder="Search SteamGridDB…"
				@keydown.enter="runSearch"
			/>
			<UiButton size="icon" variant="ghost" title="Close" @click="emit('close')">
				<X :size="16" />
			</UiButton>
		</div>

		<div
			v-if="games.length > 1"
			class="flex shrink-0 flex-wrap gap-1.5 border-b border-border px-4 py-2"
		>
			<button
				v-for="game in games"
				:key="game.id"
				type="button"
				class="cursor-pointer rounded-full border px-2.5 py-1 text-xs transition-colors"
				:class="
					selectedGame?.id === game.id
						? 'border-accent bg-accent-bg text-primary'
						: 'border-border text-secondary hover:bg-surface-4'
				"
				@click="selectGame(game)"
			>
				{{ game.name }}
			</button>
		</div>

		<div class="min-h-0 flex-1 overflow-y-auto p-4">
			<div
				v-if="searching || loadingImages"
				class="flex h-full items-center justify-center gap-2 text-sm text-secondary"
			>
				<Loader2 :size="16" class="animate-spin" />
				Loading…
			</div>
			<div v-else-if="error" class="flex h-full items-center justify-center text-sm text-danger">
				{{ error }}
			</div>
			<div
				v-else-if="images.length === 0"
				class="flex h-full items-center justify-center text-sm text-secondary"
			>
				No images found.
			</div>
			<div v-else class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3">
				<button
					v-for="image in images"
					:key="image.id"
					type="button"
					class="flex h-32 cursor-pointer items-center justify-center overflow-hidden rounded-md border border-border bg-surface-inset p-2 transition-colors hover:border-accent"
					@click="pick(image)"
				>
					<img :src="image.thumbnailUrl" alt="" class="max-h-full max-w-full object-contain" />
				</button>
			</div>
		</div>
	</Modal>
</template>
