import { invoke } from '@tauri-apps/api/core'

import type {
	ApplyResult,
	ArtworkKind,
	BackupInfo,
	DebugInfo,
	ImportCandidate,
	ImportSource,
	ManualImportRequest,
	Options,
	PersistedSettings,
	PreviewPlan,
	ScanRequest,
	ShortcutEntry,
	SteamGridDbGame,
	SteamGridDbImage,
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
	availableLaunchers: () => invoke<ImportSource[]>('available_launchers'),
	steamGridDbSearch: (apiKey: string, query: string) =>
		invoke<SteamGridDbGame[]>('steamgriddb_search', { apiKey, query }),
	steamGridDbImages: (apiKey: string, gameId: number, kind: ArtworkKind, allowNsfw: boolean) =>
		invoke<SteamGridDbImage[]>('steamgriddb_images', { apiKey, gameId, kind, allowNsfw }),
	listBackups: () => invoke<BackupInfo[]>('list_backups'),
	restoreBackup: (backupId: string) => invoke<number>('restore_backup', { backupId }),
	deleteBackup: (backupId: string) => invoke<void>('delete_backup', { backupId }),
	getDebugInfo: () => invoke<DebugInfo>('get_debug_info'),
	openLogsFolder: () => invoke<void>('open_logs_folder'),
}
