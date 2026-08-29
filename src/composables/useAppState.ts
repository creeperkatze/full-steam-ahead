import { computed, ref, watch } from 'vue'

import { api } from '../helpers/api'
import type {
	ApplyResult,
	ImportCandidate,
	ImportSource,
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
const previewVersion = ref(0)
const applyResult = ref<ApplyResult | null>(null)
const customArtwork = ref<Record<string, string>>({})
const manualPath = ref('')
const manualName = ref('')
const options = ref<Options>({
	stopSteam: false,
	restartSteam: false,
	replaceExistingArtwork: true,
	createCollections: true,
})
const steamLocation = ref('')

export interface EditableSourceSettings {
	enabled: boolean
	customPath: string
}

const availableSources = ref<ImportSource[]>([])
const sourceSettings = ref<Record<string, EditableSourceSettings>>({})

export interface EditableSteamGridDbSettings {
	enabled: boolean
	apiKey: string
	allowNsfw: boolean
}

const steamGridDb = ref<EditableSteamGridDbSettings>({
	enabled: false,
	apiKey: '',
	allowNsfw: false,
})

let settingsLoaded = false
let settingsSaveTimer: ReturnType<typeof setTimeout> | undefined

watch(
	[options, steamLocation, sourceSettings, steamGridDb],
	() => {
		if (!settingsLoaded) return
		invalidatePreview()
		clearTimeout(settingsSaveTimer)
		settingsSaveTimer = setTimeout(() => {
			api.saveSettings({
				stopSteam: options.value.stopSteam,
				restartSteam: options.value.restartSteam,
				createCollections: options.value.createCollections,
				steamLocation: steamLocation.value.trim() || null,
				sources: Object.fromEntries(
					Object.entries(sourceSettings.value).map(([key, settings]) => [
						key,
						{ enabled: settings.enabled, customPath: settings.customPath.trim() || null },
					]),
				),
				steamGridDb: {
					enabled: steamGridDb.value.enabled,
					apiKey: steamGridDb.value.apiKey.trim() || null,
					allowNsfw: steamGridDb.value.allowNsfw,
				},
			})
		}, 400)
	},
	{ deep: true },
)

let steamLocationRefreshTimer: ReturnType<typeof setTimeout> | undefined

watch(steamLocation, () => {
	clearTimeout(steamLocationRefreshTimer)
	steamLocationRefreshTimer = setTimeout(async () => {
		try {
			const detected = await api.detectSteam()
			install.value = detected
			if (!detected.users.some((user) => user.steamId === selectedUserId.value)) {
				selectedUserId.value = detected.users[0]?.steamId ?? ''
			}
		} catch {
			install.value = null
			selectedUserId.value = ''
		}
		invalidatePreview()
	}, 400)
})

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
	previewVersion.value++
}

async function loadSettingsFromDisk() {
	try {
		const [saved, sources] = await Promise.all([api.loadSettings(), api.availableSources()])
		options.value = {
			...options.value,
			stopSteam: saved.stopSteam,
			restartSteam: saved.restartSteam,
			createCollections: saved.createCollections,
		}
		steamLocation.value = saved.steamLocation ?? ''
		availableSources.value = sources
		sourceSettings.value = Object.fromEntries(
			(sources as string[]).map((key) => {
				const entry = saved.sources[key]
				return [key, { enabled: entry?.enabled ?? true, customPath: entry?.customPath ?? '' }]
			}),
		)
		steamGridDb.value = {
			enabled: saved.steamGridDb.enabled,
			apiKey: saved.steamGridDb.apiKey ?? '',
			allowNsfw: saved.steamGridDb.allowNsfw,
		}
	} catch {
		// Keep defaults
	} finally {
		settingsLoaded = true
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
		previewVersion,
		applyResult,
		customArtwork,
		manualPath,
		manualName,
		options,
		steamLocation,
		availableSources,
		sourceSettings,
		steamGridDb,
		selectedUser,
		selectedCandidates,
		usesUrlLaunch,
		toggleUrlLaunch,
		invalidatePreview,
		loadSettingsFromDisk,
	}
}
