import { invoke } from '@tauri-apps/api/core'

import type {
	ApplyResult,
	ImportCandidate,
	ManualImportRequest,
	Options,
	PreviewPlan,
	ScanRequest,
	ShortcutEntry,
	SteamInstallation,
} from '../types'

export const api = {
	detectSteam: () => invoke<SteamInstallation>('detect_steam'),
	readShortcuts: (userSteamId: string) =>
		invoke<ShortcutEntry[]>('read_shortcuts_for_user', { userSteamId }),
	scanSources: (request: ScanRequest) => invoke<ImportCandidate[]>('scan_sources', { request }),
	createManualCandidate: (request: ManualImportRequest) =>
		invoke<ImportCandidate>('create_manual_candidate', { request }),
	createPreviewPlan: (userSteamId: string, candidates: ImportCandidate[], options: Options) =>
		invoke<PreviewPlan>('create_preview_plan', { userSteamId, candidates, options }),
	applyPlan: (plan: PreviewPlan, candidates: ImportCandidate[], options: Options) =>
		invoke<ApplyResult>('apply_plan', { request: { plan, candidates, options } }),
	loadSettings: () => invoke<Pick<Options, 'stopSteam' | 'restartSteam'>>('load_settings'),
	saveSettings: (settings: Pick<Options, 'stopSteam' | 'restartSteam'>) =>
		invoke<void>('save_settings', { settings }),
}
