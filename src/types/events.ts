import type { ImportSource } from './import'

export interface ScanProgressEvent {
	source: ImportSource
	status: 'scanning' | 'done'
	found: number
}

export type ApplyStep =
	| { kind: 'stoppingSteam' }
	| { kind: 'creatingBackups' }
	| { kind: 'applyingArtwork'; gameName: string | null }
	| { kind: 'updatingShortcuts' }
	| { kind: 'updatingCollections' }
	| { kind: 'restartingSteam' }

export interface ApplyProgressEvent {
	step: ApplyStep
	current: number
	total: number
}
