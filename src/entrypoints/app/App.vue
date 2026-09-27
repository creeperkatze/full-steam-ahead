<script setup lang="ts">
import { ArrowRight, Check, Search, X } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterView, useRoute, useRouter } from 'vue-router'

import { commands } from '../../bindings'
import AppShell from '../../components/AppShell.vue'
import FlatpakPermissionModal from '../../components/FlatpakPermissionModal.vue'
import TitleBar from '../../components/TitleBar.vue'
import UiButton from '../../components/ui/Button.vue'
import { useAppState } from '../../composables/useAppState'
import { useReviewPlan } from '../../composables/useReviewPlan'
import { useScanSources } from '../../composables/useScanSources'
import { useTaskStatus } from '../../composables/useTaskStatus'

const router = useRouter()
const route = useRoute()
const state = useAppState()
const { t } = useI18n()

const showFlatpakPermissionModal = ref(false)

onMounted(() => state.loadSettingsFromDisk())
const reviewPlan = useReviewPlan()
const task = useTaskStatus()
const { scan: scanSources } = useScanSources()
const settingsOpen = computed(() => route.path.startsWith('/settings'))

const activeStepIndex = computed(() => {
	if (state.step.value === 'sources') return 1
	if (state.step.value === 'artwork') return 2
	if (state.step.value === 'review') return 3
	if (state.step.value === 'done') return 4
	return 0
})

const navigableSteps = computed(() => [
	true,
	state.scanPhase.value === 'done',
	state.selectedCandidates.value.length > 0,
	state.selectedCandidates.value.length > 0,
	activeStepIndex.value >= 4,
])

const nextLabel = computed(() => {
	if (state.step.value === 'review') return t('app.actions.apply')
	return t('app.actions.continue')
})

const nextDisabled = computed(() => {
	if (task.loading.value) return true
	if (state.step.value === 'sources') return state.selectedCandidates.value.length === 0
	if (state.step.value === 'review') return !state.previewPlan.value
	return false
})

const scanDisabled = computed(
	() => task.loading.value || !state.selectedUser.value || state.scanPhase.value === 'scanning',
)

const showActionBar = computed(() => !settingsOpen.value)

function toggleSettings() {
	router.push(settingsOpen.value ? '/' : '/settings')
}

async function toStepIndex(index: number) {
	if (index === 0) {
		state.step.value = 'start'
		return
	}

	if (index === 1) {
		if (state.scanPhase.value === 'done') {
			state.step.value = 'sources'
		}
		return
	}

	if (index === 2) {
		if (state.selectedCandidates.value.length > 0) {
			state.step.value = 'artwork'
		}
		return
	}

	if (index === 3 && state.selectedCandidates.value.length > 0) {
		if (state.previewPlan.value) {
			state.step.value = 'review'
			return
		}
		await reviewPlan.createPreview()
	}
}

function back() {
	if (state.step.value === 'sources') {
		state.step.value = 'start'
	} else if (state.step.value === 'artwork') {
		state.step.value = 'sources'
	} else if (state.step.value === 'review') {
		state.step.value = 'artwork'
	} else if (state.step.value === 'done') {
		state.step.value = 'review'
	}
}

async function scan() {
	await scanSources()
	if (state.scanPhase.value === 'done') {
		state.step.value = 'sources'
	}
}

function continueToSources() {
	state.step.value = 'sources'
}

async function next() {
	if (state.step.value === 'sources') {
		state.step.value = 'artwork'
		return
	}

	if (state.step.value === 'artwork') {
		await reviewPlan.createPreview()
		return
	}

	if (state.install.value?.needsFlatpakPermission) {
		showFlatpakPermissionModal.value = true
		return
	}

	await apply()
}

async function apply() {
	state.step.value = 'done'
	await reviewPlan.applyPreview()
}

async function onFlatpakPermissionModalClosed() {
	showFlatpakPermissionModal.value = false
	await apply()
}
</script>

<template>
	<div class="flex h-screen flex-col bg-surface-2">
		<TitleBar
			:active-step="activeStepIndex"
			:navigable-steps="navigableSteps"
			:settings-open="settingsOpen"
			@select-step="toStepIndex"
			@toggle-settings="toggleSettings"
		/>
		<AppShell>
			<RouterView />

			<template #footer>
				<div v-if="showActionBar" class="flex shrink-0 justify-center px-2">
					<div class="flex items-center gap-2">
						<UiButton v-if="state.step.value !== 'start'" @click="back">
							{{ t('app.actions.back') }}
						</UiButton>

						<template v-if="state.step.value === 'start'">
							<UiButton
								:variant="state.scanPhase.value === 'done' ? 'default' : 'primary'"
								:disabled="scanDisabled"
								@click="scan"
							>
								{{ t('app.actions.scan') }}
								<template #icon><Search :size="16" /></template>
							</UiButton>
							<UiButton
								v-if="state.scanPhase.value === 'done'"
								variant="primary"
								@click="continueToSources"
							>
								{{ t('app.actions.continue') }}
								<template #icon><ArrowRight :size="16" /></template>
							</UiButton>
						</template>
						<UiButton
							v-else-if="state.step.value !== 'done'"
							variant="primary"
							:disabled="nextDisabled"
							@click="next"
						>
							{{ nextLabel }}
							<template #icon>
								<Check v-if="state.step.value === 'review'" :size="18" />
								<ArrowRight v-else :size="16" />
							</template>
						</UiButton>

						<UiButton
							v-else-if="state.step.value === 'done' && !task.loading.value"
							variant="primary"
							@click="commands.closeApp()"
						>
							{{ t('app.actions.close') }}
							<template #icon><X :size="16" /></template>
						</UiButton>
					</div>
				</div>
			</template>
		</AppShell>
	</div>

	<FlatpakPermissionModal
		:model-value="showFlatpakPermissionModal"
		@update:model-value="onFlatpakPermissionModalClosed"
	/>
</template>
