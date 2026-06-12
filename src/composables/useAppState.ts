import { computed, ref, watch } from 'vue'

import { api } from '../helpers/api'
import type {
	ApplyResult,
	ImportCandidate,
	Options,
	PreviewPlan,
	SteamInstallation,
	SteamUser,
} from '../types'

export type FlowStep = 'start' | 'sources' | 'artwork' | 'review' | 'done'
export type ScanPhase = 'idle' | 'scanning' | 'done'

const step = ref<FlowStep>('start')
const scanPhase = ref<ScanPhase>('idle')
const install = ref<SteamInstallation | null>(null)
const selectedUserId = ref('')
const candidates = ref<ImportCandidate[]>([])
const selectedCandidateIds = ref<Set<string>>(new Set())
const previewPlan = ref<PreviewPlan | null>(null)
const applyResult = ref<ApplyResult | null>(null)
const customArtwork = ref<Record<string, string>>({})
const manualPath = ref('')
const manualName = ref('')
const options = ref<Options>({
	stopSteam: false,
	restartSteam: false,
	replaceExistingArtwork: true,
})

watch(
	options,
	() => {
		api.saveSettings({
			stopSteam: options.value.stopSteam,
			restartSteam: options.value.restartSteam,
		})
		invalidatePreview()
	},
	{ deep: true },
)

const selectedUser = computed<SteamUser | undefined>(() =>
	install.value?.users.find((user) => user.steamId === selectedUserId.value),
)

const selectedCandidates = computed(() =>
	candidates.value.filter((candidate) => selectedCandidateIds.value.has(candidate.id)),
)

function usesUrlLaunch(candidate: ImportCandidate): boolean {
	if (!candidate.urlScheme) return false
	if (!candidate.launcherPath) return true
	return candidate.useLauncherUrl
}

function toggleUrlLaunch(id: string) {
	const idx = candidates.value.findIndex((c) => c.id === id)
	if (idx === -1) return
	candidates.value[idx] = {
		...candidates.value[idx],
		useLauncherUrl: !candidates.value[idx].useLauncherUrl,
	}
	invalidatePreview()
}

function invalidatePreview() {
	previewPlan.value = null
	applyResult.value = null
}

async function loadSettingsFromDisk() {
	try {
		const saved = await api.loadSettings()
		options.value = { ...options.value, ...saved }
	} catch {
		// Keep defaults
	}
}

export function useAppState() {
	return {
		step,
		scanPhase,
		install,
		selectedUserId,
		candidates,
		selectedCandidateIds,
		previewPlan,
		applyResult,
		customArtwork,
		manualPath,
		manualName,
		options,
		selectedUser,
		selectedCandidates,
		usesUrlLaunch,
		toggleUrlLaunch,
		invalidatePreview,
		loadSettingsFromDisk,
	}
}
