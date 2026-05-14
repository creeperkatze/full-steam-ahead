<script setup lang="ts">
import { computed } from 'vue'

import { useAppState } from '../composables/useAppState'
import { importSourceName } from '../helpers/sourceNames'
import type { ImportCandidate } from '../types'
import GameIcon from './GameIcon.vue'
import SourceIcon from './SourceIcon.vue'
import ItemRow from './ui/ItemRow.vue'
import Toggle from './ui/Toggle.vue'

const state = useAppState()

const props = defineProps<{
	title: string
	source?: string
	candidates: ImportCandidate[]
	selectedIds: Set<string>
	showSource?: boolean
}>()

const emit = defineEmits<{
	toggle: [id: string]
	'set-all': [value: boolean]
}>()

const selectedCount = computed(
	() => props.candidates.filter((c) => props.selectedIds.has(c.id)).length,
)

const allSelected = computed(
	() => props.candidates.length > 0 && selectedCount.value === props.candidates.length,
)
</script>

<template>
	<article class="overflow-hidden rounded-xl border border-border">
		<label
			class="flex cursor-pointer items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5"
		>
			<input
				type="checkbox"
				:checked="allSelected"
				@change="emit('set-all', ($event.target as HTMLInputElement).checked)"
			/>
			<SourceIcon v-if="source" :source="source" class="size-5 shrink-0" />
			<strong class="min-w-0 flex-1 truncate text-base">{{ title }}</strong>
			<span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
				{{ selectedCount }} / {{ candidates.length }}
			</span>
		</label>

		<div class="grid gap-1.5 bg-surface-3 p-2">
			<ItemRow v-for="candidate in candidates" :key="candidate.id" as="label" interactive>
				<template #leading>
					<input
						type="checkbox"
						:checked="selectedIds.has(candidate.id)"
						@change="emit('toggle', candidate.id)"
					/>
					<GameIcon :candidate="candidate" :size="20" />
				</template>

				<strong class="block truncate">{{ candidate.name }}</strong>
				<small class="block text-secondary/70">{{ candidate.executablePath }}</small>
				<small v-if="showSource" class="block text-secondary">{{
					importSourceName(candidate.source)
				}}</small>

				<template #trailing>
					<div
						v-if="candidate.urlScheme && !showSource"
						class="flex shrink-0 items-center gap-1.5"
						:title="
							!candidate.launcherPath
								? 'This game can only be launched via its URL scheme'
								: undefined
						"
					>
						<span class="text-xs text-secondary">Via Launcher</span>
						<Toggle
							:model-value="state.usesUrlLaunch(candidate)"
							:disabled="!candidate.launcherPath"
							@update:model-value="state.toggleUrlLaunch(candidate.id)"
						/>
					</div>
				</template>
			</ItemRow>
		</div>
	</article>
</template>
