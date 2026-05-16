export interface ScanProgressEvent {
	source: string
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
