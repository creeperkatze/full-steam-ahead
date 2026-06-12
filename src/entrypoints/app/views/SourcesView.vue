<script setup lang="ts">
import { FolderPlus, Plus } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { computed } from 'vue'

import GameIcon from '../../../components/GameIcon.vue'
import SourceCard from '../../../components/SourceCard.vue'
import Checkbox from '../../../components/ui/Checkbox.vue'
import ItemRow from '../../../components/ui/ItemRow.vue'
import UiButton from '../../../components/ui/Button.vue'
import { useAppState } from '../../../composables/useAppState'
import { SCANNABLE_SOURCES } from '../../../composables/useScanSources'
import { useTaskStatus } from '../../../composables/useTaskStatus'
import { api } from '../../../helpers/api'
import { importSourceName } from '../../../helpers/sourceNames'
import type { ImportCandidate, ImportSource } from '../../../types'

const state = useAppState()
const task = useTaskStatus()

interface PlatformCard {
	key: string
	title: string
	candidates: ImportCandidate[]
}

const platformCards = computed<PlatformCard[]>(() =>
	SCANNABLE_SOURCES.map((source) => ({
		key: source,
		title: importSourceName(source),
		candidates: candidatesFor(source),
	})).filter((card) => card.candidates.length > 0),
)

const manualCandidates = computed(() => candidatesFor('manual'))
const otherCards = computed(() => {
	const grouped = new Map<string, ImportCandidate[]>()
	for (const candidate of state.candidates.value) {
		if (typeof candidate.source !== 'string') {
			const label = candidate.source.other
			grouped.set(label, [...(grouped.get(label) ?? []), candidate])
		}
	}
	return Array.from(grouped.entries()).map(([title, candidates]) => ({ title, candidates }))
})

function candidatesFor(source: ImportSource) {
	return state.candidates.value.filter((candidate) => candidate.source === source)
}

function selectedIn(candidates: ImportCandidate[]) {
	return candidates.filter((c) => state.selectedCandidateIds.value.has(c.id)).length
}

function allSelected(candidates: ImportCandidate[]) {
	return candidates.length > 0 && selectedIn(candidates) === candidates.length
}

function setCandidatesSelected(candidates: ImportCandidate[], value: boolean) {
	for (const candidate of candidates) {
		if (state.selectedCandidateIds.value.has(candidate.id) !== value) {
			toggleCandidate(candidate.id)
		}
	}
}

async function pickExecutable() {
	const picked = await open({
		multiple: false,
		filters: [{ name: 'Executable', extensions: ['exe', 'bat', 'cmd'] }],
	})
	if (typeof picked === 'string') {
		state.manualPath.value = picked
	}
}

async function addManual() {
	if (!state.selectedUserId.value || !state.manualPath.value.trim()) return

	const candidate = await task.runTask('Adding manual entry', () =>
		api.createManualCandidate({
			userSteamId: state.selectedUserId.value,
			executablePath: state.manualPath.value.trim(),
			displayName: state.manualName.value.trim() || undefined,
			source: 'manual',
			tags: ['Manual'],
		}),
	)
	if (!candidate) return

	state.candidates.value = [...state.candidates.value, candidate].sort((a, b) =>
		a.name.localeCompare(b.name),
	)
	state.selectedCandidateIds.value = new Set([...state.selectedCandidateIds.value, candidate.id])
	state.manualPath.value = ''
	state.manualName.value = ''
	state.invalidatePreview()
}

function toggleCandidate(id: string) {
	const next = new Set(state.selectedCandidateIds.value)
	if (next.has(id)) {
		next.delete(id)
	} else {
		next.add(id)
	}
	state.selectedCandidateIds.value = next
	state.invalidatePreview()
}
</script>

<template>
	<div class="flex flex-1 flex-col gap-4">
		<!-- ── Source cards ───────────────────────────────────────────── -->
		<section class="grid gap-3">
			<SourceCard
				v-for="card in platformCards"
				:key="card.key"
				:title="card.title"
				:source="card.key"
				:candidates="card.candidates"
				:selected-ids="state.selectedCandidateIds.value"
				@toggle="toggleCandidate"
				@set-all="setCandidatesSelected(card.candidates, $event)"
			/>

			<SourceCard
				v-for="card in otherCards"
				:key="card.title"
				:title="card.title"
				:candidates="card.candidates"
				:selected-ids="state.selectedCandidateIds.value"
				show-source
				@toggle="toggleCandidate"
				@set-all="setCandidatesSelected(card.candidates, $event)"
			/>
		</section>

		<!-- ── Manual section ────────────────────────────────────────── -->
		<section class="overflow-hidden rounded-xl border border-border">
			<label
				class="flex cursor-pointer items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5"
			>
				<Checkbox
					:model-value="allSelected(manualCandidates)"
					:disabled="manualCandidates.length === 0"
					@update:model-value="setCandidatesSelected(manualCandidates, $event)"
				/>
				<strong class="min-w-0 flex-1 truncate text-base">{{ importSourceName('manual') }}</strong>
				<span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
					{{ selectedIn(manualCandidates) }} / {{ manualCandidates.length }}
				</span>
			</label>

			<div class="grid gap-1.5 bg-surface-3 p-2">
				<div
					class="flex items-center gap-2 rounded-lg border border-border/60 bg-surface-5 px-3 py-2"
				>
					<UiButton size="icon" variant="secondary" title="Pick executable" @click="pickExecutable">
						<FolderPlus :size="18" />
					</UiButton>
					<input
						v-model="state.manualPath.value"
						class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-3 px-2 text-primary"
						placeholder="Executable path"
					/>
					<input
						v-model="state.manualName.value"
						class="h-9 w-64 rounded-md border border-border bg-surface-3 px-2 text-primary"
						placeholder="Display name"
					/>
					<UiButton variant="secondary" :disabled="!state.manualPath.value" @click="addManual">
						Add
						<template #icon><Plus :size="20" /></template>
					</UiButton>
				</div>

				<ItemRow v-for="candidate in manualCandidates" :key="candidate.id" as="label" interactive>
					<template #leading>
						<Checkbox
							:model-value="state.selectedCandidateIds.value.has(candidate.id)"
							@update:model-value="toggleCandidate(candidate.id)"
						/>
						<GameIcon :candidate="candidate" :size="20" />
					</template>
					<strong class="block truncate">{{ candidate.name }}</strong>
					<small class="block text-secondary/70">{{ candidate.executablePath }}</small>
				</ItemRow>

				<div
					v-if="manualCandidates.length === 0"
					class="grid min-h-20 place-items-center rounded-lg border border-dashed border-border-dashed p-4 text-center text-sm text-secondary"
				>
					No manual games added yet.
				</div>
			</div>
		</section>
	</div>
</template>
