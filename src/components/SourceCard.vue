<script setup lang="ts">
import { Pencil, RotateCcw } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { useAppState } from '../composables/useAppState'
import { importSourceName } from '../helpers/sourceNames'
import type { ImportCandidate } from '../types'
import GameIcon from './GameIcon.vue'
import SourceIcon from './SourceIcon.vue'
import UiButton from './ui/Button.vue'
import Checkbox from './ui/Checkbox.vue'
import ItemRow from './ui/ItemRow.vue'
import Toggle from './ui/Toggle.vue'

const state = useAppState()
const { t } = useI18n()

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

defineSlots<{
	'before-list'?: () => unknown
	empty?: () => unknown
}>()

const selectedCount = computed(
	() => props.candidates.filter((c) => props.selectedIds.has(c.id)).length,
)

const allSelected = computed(
	() => props.candidates.length > 0 && selectedCount.value === props.candidates.length,
)

const someSelected = computed(() => selectedCount.value > 0 && !allSelected.value)

const renamingId = ref<string | null>(null)
const renameDraft = ref('')

function startRename(candidate: ImportCandidate) {
	renamingId.value = candidate.id
	renameDraft.value = candidate.name
}

function commitRename() {
	if (renamingId.value === null) return
	state.renameCandidate(renamingId.value, renameDraft.value)
	renamingId.value = null
}

function cancelRename() {
	renamingId.value = null
}

// Called on every render, so only the first call may move the focus.
function focusInput(el: unknown) {
	if (el instanceof HTMLInputElement && document.activeElement !== el) {
		el.focus()
		el.select()
	}
}
</script>

<template>
	<article class="overflow-hidden rounded-lg border border-border">
		<label
			class="flex cursor-pointer items-center gap-3 border-b border-border bg-surface-4 px-3 py-2.5 transition-colors hover:bg-surface-hover"
		>
			<Checkbox
				:model-value="allSelected"
				:neutral="someSelected"
				:disabled="candidates.length === 0"
				@update:model-value="emit('set-all', $event)"
			/>
			<SourceIcon v-if="source" :source="source" class="size-5 shrink-0" />
			<strong class="min-w-0 flex-1 truncate text-base">{{ title }}</strong>
			<span class="shrink-0 rounded-md border border-border px-2 py-1 text-xs text-secondary">
				{{ selectedCount }} / {{ candidates.length }}
			</span>
		</label>

		<div class="grid gap-1.5 bg-surface-3 p-2">
			<slot name="before-list" />

			<ItemRow v-for="candidate in candidates" :key="candidate.id" as="label" interactive>
				<template #leading>
					<Checkbox
						:model-value="selectedIds.has(candidate.id)"
						@update:model-value="emit('toggle', candidate.id)"
					/>
					<GameIcon :candidate="candidate" :size="20" />
				</template>

				<input
					v-if="renamingId === candidate.id"
					:ref="focusInput"
					v-model="renameDraft"
					class="block h-7 w-full rounded-md border border-border bg-surface-3 px-2 font-bold text-primary"
					@keydown.enter.prevent="commitRename"
					@keydown.esc.prevent="cancelRename"
					@blur="commitRename"
				/>
				<strong v-else class="block truncate">{{ candidate.name }}</strong>
				<small class="block text-secondary/70">{{ candidate.executablePath }}</small>
				<small v-if="showSource" class="block text-secondary">{{
					importSourceName(candidate.source)
				}}</small>

				<template #trailing>
					<div class="flex shrink-0 items-center gap-2">
						<div
							v-if="candidate.urlScheme && !showSource"
							class="flex items-center gap-1.5"
							:title="!candidate.launcherPath ? t('sourceCard.urlOnlyTitle') : undefined"
						>
							<span class="text-xs text-secondary">{{ t('sourceCard.viaLauncher') }}</span>
							<Toggle
								:model-value="state.usesUrlLaunch(candidate)"
								:disabled="!candidate.launcherPath"
								@update:model-value="state.toggleUrlLaunch(candidate.id)"
							/>
						</div>
						<UiButton
							class="h-8 w-8"
							:disabled="renamingId === candidate.id"
							size="icon"
							variant="ghost"
							:title="t('sourceCard.rename')"
							@click.prevent="startRename(candidate)"
						>
							<Pencil :size="14" />
						</UiButton>
						<UiButton
							class="h-8 w-8"
							size="icon"
							variant="ghost"
							:title="t('sourceCard.resetName')"
							:disabled="candidate.name === candidate.originalName"
							@click.prevent="state.renameCandidate(candidate.id, candidate.originalName)"
						>
							<RotateCcw :size="14" />
						</UiButton>
					</div>
				</template>
			</ItemRow>

			<slot v-if="candidates.length === 0" name="empty" />
		</div>
	</article>
</template>
