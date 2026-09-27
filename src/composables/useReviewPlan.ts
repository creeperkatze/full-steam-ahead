import { ref, watch } from 'vue'

import { commands, events } from '../bindings'
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
				commands.createPreviewPlan(
					state.selectedUserId.value,
					state.selectedCandidates.value,
					state.settings,
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
	const unlisten = await events.applyProgressEvent.listen((event) => {
		applyProgress.value = event.payload
	})

	const result = await task.runTask('Applying changes', () =>
		commands.applyPlan({
			plan: state.previewPlan.value!,
			candidates: state.selectedCandidates.value,
			options: state.settings,
		}),
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
