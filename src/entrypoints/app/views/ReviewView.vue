<script setup lang="ts">
import { ChevronDown, FolderArchive, Image, Library, ListChecks } from '@lucide/vue'
import { computed } from 'vue'

import GameIcon from '../../../components/GameIcon.vue'
import ItemRow from '../../../components/ui/ItemRow.vue'
import { useAppState } from '../../../composables/useAppState'
import type { PlannedChange, PreviewPlan } from '../../../types'

const state = useAppState()

const props = defineProps<{
	plan: PreviewPlan | null
}>()

interface ArtworkChange {
	kind: string
	source: string
	destructive: boolean
}

interface CollectionChange {
	name: string
	destructive: boolean
}

interface GameReview {
	name: string
	shortcut?: PlannedChange
	collections: CollectionChange[]
	artwork: ArtworkChange[]
}

const games = computed(() => {
	const grouped = new Map<string, GameReview>()

	for (const change of props.plan?.changes ?? []) {
		const name = change.gameName
		if (!name) continue

		const game = grouped.get(name) ?? { name, collections: [], artwork: [] }

		if (change.kind === 'addShortcut' || change.kind === 'updateShortcut') {
			game.shortcut = change
		} else if (change.kind === 'updateCollections') {
			game.collections.push({
				name: change.collectionName ?? 'Managed',
				destructive: change.destructive,
			})
		} else if (change.kind === 'writeArtwork') {
			game.artwork.push({
				kind: titleCase(change.artworkKind ?? 'artwork'),
				source: sourceLabel(change.artworkSource ?? ''),
				destructive: change.destructive,
			})
		}

		grouped.set(name, game)
	}

	return Array.from(grouped.values()).sort((a, b) => a.name.localeCompare(b.name))
})

const candidateByName = computed(() => new Map(state.candidates.value.map((c) => [c.name, c])))

function changeCount(game: GameReview) {
	return Number(Boolean(game.shortcut)) + game.collections.length + game.artwork.length
}

function sourceLabel(source: string) {
	switch (source.toLowerCase()) {
		case 'officialsteam':
			return 'Official Steam'
		case 'steamgriddb':
			return 'SteamGridDB'
		case 'localfile':
			return 'Local file'
		case 'existingcustom':
			return 'Existing'
		default:
			return source || 'Unknown'
	}
}

function titleCase(value: string) {
	return value.charAt(0).toUpperCase() + value.slice(1)
}

function fileName(path: string) {
	const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
	return lastSep >= 0 ? path.slice(lastSep + 1) : path
}
</script>

<template>
	<!-- Review state -->
	<section class="grid gap-3">
		<div
			v-if="!plan"
			class="grid min-h-55 place-items-center rounded-lg border border-border bg-surface-3 p-6 text-secondary"
		>
			Preparing preview...
		</div>

		<template v-else>
			<!-- Game list -->
			<div class="grid gap-2">
				<article
					v-for="game in games"
					:key="game.name"
					class="overflow-hidden rounded-xl border border-border"
				>
					<div class="flex items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5">
						<GameIcon
							v-if="candidateByName.get(game.name)"
							:candidate="candidateByName.get(game.name)!"
							:size="20"
						/>
						<strong class="min-w-0 flex-1 truncate text-base">{{ game.name }}</strong>
						<span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
							{{ changeCount(game) }} change{{ changeCount(game) === 1 ? '' : 's' }}
						</span>
					</div>

					<div class="grid gap-1.5 bg-surface-3 p-2">
						<ItemRow v-if="game.shortcut">
							<template #leading>
								<ListChecks :size="15" class="text-accent" />
							</template>
							Steam entry
							<template #trailing>
								<span
									v-if="game.shortcut.kind === 'addShortcut'"
									class="shrink-0 text-xs text-accent"
									>New</span
								>
								<span v-else class="shrink-0 text-xs text-secondary">Update</span>
							</template>
						</ItemRow>

						<ItemRow v-for="asset in game.artwork" :key="`${game.name}:${asset.kind}`">
							<template #leading>
								<Image :size="15" class="text-accent" />
							</template>
							<strong>{{ asset.kind }}</strong>
							<span class="text-secondary"> · {{ asset.source }}</span>
							<template #trailing>
								<span v-if="asset.destructive" class="shrink-0 text-xs text-secondary">Update</span>
								<span v-else class="shrink-0 text-xs text-accent">New</span>
							</template>
						</ItemRow>

						<ItemRow v-for="coll in game.collections" :key="`${game.name}:coll:${coll.name}`">
							<template #leading>
								<Library :size="15" class="text-accent" />
							</template>
							{{ coll.name }}
							<template #trailing>
								<span v-if="coll.destructive" class="shrink-0 text-xs text-secondary"
									>Already added</span
								>
								<span v-else class="shrink-0 text-xs text-accent">Add</span>
							</template>
						</ItemRow>
					</div>
				</article>
			</div>

			<!-- Backup details -->
			<details class="group overflow-hidden rounded-xl border border-border">
				<summary
					class="flex cursor-pointer list-none items-center justify-between gap-3 border-b border-transparent bg-surface-4 px-3 py-2.5 text-sm group-open:border-border"
				>
					<span class="inline-flex items-center gap-2">
						<FolderArchive :size="15" />
						<strong>Backups</strong>
					</span>
					<span class="flex items-center gap-2">
						<span class="rounded-md border border-border px-2 py-1 text-xs text-secondary"
							>{{ plan.backups.length }} files</span
						>
						<ChevronDown
							:size="14"
							class="text-secondary transition-transform group-open:rotate-180"
						/>
					</span>
				</summary>
				<div class="grid gap-1.5 bg-surface-3 p-2">
					<ItemRow v-for="backup in plan.backups" :key="backup.destination">
						<template #leading>
							<FolderArchive :size="14" class="shrink-0 text-accent" />
						</template>
						<span class="truncate">{{ fileName(backup.source) }}</span>
					</ItemRow>
				</div>
			</details>
		</template>
	</section>
</template>
