<script setup lang="ts">
import { Check, Loader2 } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import type { ApplyProgressEvent, ApplyResult, ApplyStep } from '../../../types'

defineProps<{
	applyResult: ApplyResult | null
	applyProgress: ApplyProgressEvent | null
}>()

const { t } = useI18n()

function stepLabel(step: ApplyStep): string {
	switch (step.kind) {
		case 'stoppingSteam':
			return t('doneView.steps.stoppingSteam')
		case 'creatingBackups':
			return t('doneView.steps.creatingBackups')
		case 'applyingArtwork':
			return step.gameName
				? t('doneView.steps.applyingArtworkFor', { name: step.gameName })
				: t('doneView.steps.applyingArtwork')
		case 'updatingShortcuts':
			return t('doneView.steps.updatingShortcuts')
		case 'updatingCollections':
			return t('doneView.steps.updatingCollections')
		case 'restartingSteam':
			return t('doneView.steps.restartingSteam')
	}
}
</script>

<template>
	<div class="flex flex-1 flex-col">
		<!-- Apply progress -->
		<section
			v-if="applyProgress"
			class="flex flex-1 flex-col items-center justify-center gap-4 rounded-lg border border-accent/30 bg-accent-bg p-8"
		>
			<div class="w-full max-w-sm">
				<div class="mb-3 flex items-center gap-3">
					<Loader2 :size="18" class="shrink-0 animate-spin text-accent" />
					<strong class="min-w-0 flex-1 truncate">{{ stepLabel(applyProgress.step) }}</strong>
					<span class="shrink-0 text-xs text-secondary"
						>{{ applyProgress.current }} / {{ applyProgress.total }}</span
					>
				</div>
				<div class="h-2 overflow-hidden rounded-full bg-surface-3">
					<div
						class="h-full rounded-full bg-accent transition-all duration-300"
						:style="{ width: `${(applyProgress.current / applyProgress.total) * 100}%` }"
					/>
				</div>
			</div>
		</section>

		<!-- Success state -->
		<section
			v-else-if="applyResult"
			class="flex flex-1 flex-col items-center justify-center gap-6 rounded-lg border border-accent/30 bg-accent-bg text-center"
		>
			<div class="grid size-14 place-items-center rounded-full bg-accent text-accent-icon">
				<Check :size="28" />
			</div>
			<div>
				<h1 class="text-2xl font-bold">{{ t('doneView.allDoneTitle') }}</h1>
				<p class="mt-1 text-secondary">{{ t('doneView.allDoneSubtitle') }}</p>
			</div>
			<div class="flex items-center gap-6 rounded-lg border border-border bg-surface-3 px-6 py-3">
				<div class="text-center">
					<strong class="block text-2xl">{{ applyResult.appliedChanges.length }}</strong>
					<span class="text-xs text-secondary">{{ t('doneView.changesApplied') }}</span>
				</div>
				<div class="h-10 w-px bg-border" />
				<div class="text-center">
					<strong class="block text-2xl">{{ applyResult.backupsCreated.length }}</strong>
					<span class="text-xs text-secondary">{{ t('doneView.backupsCreated') }}</span>
				</div>
			</div>
		</section>

		<!-- Initial loading state -->
		<section
			v-else
			class="flex flex-1 items-center justify-center gap-3 rounded-lg border border-border bg-surface-3 text-secondary"
		>
			<Loader2 :size="18" class="animate-spin text-accent" />
			<span>{{ t('doneView.starting') }}</span>
		</section>
	</div>
</template>
