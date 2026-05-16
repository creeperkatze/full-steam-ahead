export interface Options {
	stopSteam: boolean
	restartSteam: boolean
	replaceExistingArtwork: boolean
}

export interface PreviewPlan {
	userSteamId: string
	changes: PlannedChange[]
	filesToChange: string[]
	backups: BackupPlan[]
	requiresSteamRestart: boolean
}

export interface BackupPlan {
	source: string
	destination: string
}

import type { ArtworkKind, ArtworkSource } from './import'

export interface PlannedChange {
	id: string
	gameName: string
	file: string
	kind: 'addShortcut' | 'updateShortcut' | 'writeArtwork' | 'updateCollections'
	destructive: boolean
	artworkSource: ArtworkSource | null
	artworkKind: ArtworkKind | null
	collectionName: string | null
}

export interface ApplyResult {
	appliedChanges: PlannedChange[]
	backupsCreated: string[]
}

export interface BackupInfo {
	id: string
	createdAt: string
	fileCount: number
	sizeBytes: number
}
