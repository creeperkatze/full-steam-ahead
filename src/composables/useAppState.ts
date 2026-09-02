import { computed, reactive, ref, watch } from 'vue'

import { api } from '../helpers/api'
import { applyLocale, type SupportedLocale } from '../i18n'
import { applyColorScheme, type ColorScheme } from '../theme'
import type {
	ApplyResult,
	ImportCandidate,
	ImportSource,
	PreviewPlan,
	Settings,
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

const availableSources = ref<ImportSource[]>([])

const settings = reactive<Settings>({} as Settings)
const settingsReady = ref(false)

let settingsSaveTimer: ReturnType<typeof setTimeout> | undefined

async function flushSettingsSave() {
	if (!settingsReady.value || !settingsSaveTimer) return
	clearTimeout(settingsSaveTimer)
	settingsSaveTimer = undefined
	await api.saveSettings({ ...settings })
}

document.addEventListener('visibilitychange', () => {
	if (document.visibilityState === 'hidden') void flushSettingsSave()
})

watch(
	settings,
	() => {
		if (!settingsReady.value) return
		invalidatePreview()
		clearTimeout(settingsSaveTimer)
		settingsSaveTimer = setTimeout(() => {
			settingsSaveTimer = undefined
			void api.saveSettings({ ...settings })
		}, 400)
	},
	{ deep: true },
)

watch(
	() => settings.locale,
	(value) => applyLocale(value as SupportedLocale | null),
)
watch(
	() => settings.colorScheme,
	(value) => applyColorScheme(value as ColorScheme | null),
)

let steamLocationRefreshTimer: ReturnType<typeof setTimeout> | undefined

watch(
	() => settings.steamLocation,
	() => {
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
	},
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
	previewVersion.value++
}

function applySettings(newSettings: Settings) {
	Object.assign(settings, newSettings, {
		sources: Object.fromEntries(
			(availableSources.value as string[]).map((key) => [
				key,
				newSettings.sources[key] ?? { enabled: true, customPath: null },
			]),
		),
	})
}

async function loadSettingsFromDisk() {
	try {
		const [saved, sources] = await Promise.all([api.loadSettings(), api.availableSources()])
		availableSources.value = sources
		applySettings(saved)
	} catch {
		// Leave settings empty
	} finally {
		settingsReady.value = true
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
		settings,
		settingsReady,
		availableSources,
		selectedUser,
		selectedCandidates,
		usesUrlLaunch,
		toggleUrlLaunch,
		invalidatePreview,
		loadSettingsFromDisk,
		applySettings,
	}
}
