import { listen } from '@tauri-apps/api/event'
import { ref, watch } from 'vue'

import { api } from '../helpers/api'
import type { ApplyProgressEvent } from '../types'
import { useAppState } from './useAppState'
import { useTaskStatus } from './useTaskStatus'

const state = useAppState()
const task = useTaskStatus()

const applyProgress = ref<ApplyProgressEvent | null>(null)

let regenerating = false

async function createPreview() {
	if (!state.selectedUserId.value) return false
	if (regenerating) return false

	regenerating = true
	try {
		for (;;) {
			const versionAtStart = state.previewVersion.value
			const plan = await task.runTask('Creating preview', () =>
				api.createPreviewPlan(
					state.selectedUserId.value,
					state.selectedCandidates.value,
					state.options.value,
				),
			)
			if (!plan) return false
			if (state.previewVersion.value !== versionAtStart) continue

			state.previewPlan.value = plan
			state.applyResult.value = null
			state.step.value = 'review'
			return true
		}
	} finally {
		regenerating = false
	}
}

watch([state.previewPlan, state.step], ([plan, step]) => {
	if (plan === null && step === 'review') void createPreview()
})

async function applyPreview() {
	if (!state.previewPlan.value) return

	applyProgress.value = null
	const unlisten = await listen<ApplyProgressEvent>('apply-progress', (event) => {
		applyProgress.value = event.payload
	})

	const result = await task.runTask('Applying changes', () =>
		api.applyPlan(state.previewPlan.value!, state.selectedCandidates.value, state.options.value),
	)

	unlisten()
	applyProgress.value = null

	if (result) {
		state.applyResult.value = result
	}
}

export function useReviewPlan() {
	return {
		createPreview,
		applyPreview,
		applyProgress,
	}
}
