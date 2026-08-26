import { invoke } from '@tauri-apps/api/core'

import type {
	ApplyResult,
	BackupInfo,
	ImportCandidate,
	ManualImportRequest,
	Options,
	PersistedSettings,
	PreviewPlan,
	ScanRequest,
	ShortcutEntry,
	SteamInstallation,
} from '../types'

export const api = {
	detectSteam: () => invoke<SteamInstallation>('detect_steam'),
	validateSteamLocation: (path: string) => invoke<boolean>('validate_steam_location', { path }),
	readShortcuts: (userSteamId: string) =>
		invoke<ShortcutEntry[]>('read_shortcuts_for_user', { userSteamId }),
	scanSources: (request: ScanRequest) => invoke<ImportCandidate[]>('scan_sources', { request }),
	createManualCandidate: (request: ManualImportRequest) =>
		invoke<ImportCandidate>('create_manual_candidate', { request }),
	createPreviewPlan: (userSteamId: string, candidates: ImportCandidate[], options: Options) =>
		invoke<PreviewPlan>('create_preview_plan', { userSteamId, candidates, options }),
	applyPlan: (plan: PreviewPlan, candidates: ImportCandidate[], options: Options) =>
		invoke<ApplyResult>('apply_plan', { request: { plan, candidates, options } }),
	loadSettings: () => invoke<PersistedSettings>('load_settings'),
	saveSettings: (settings: PersistedSettings) => invoke<void>('save_settings', { settings }),
	listBackups: () => invoke<BackupInfo[]>('list_backups'),
	restoreBackup: (backupId: string) => invoke<number>('restore_backup', { backupId }),
	deleteBackup: (backupId: string) => invoke<void>('delete_backup', { backupId }),
}
