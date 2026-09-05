import { listen } from '@tauri-apps/api/event'
import { ref } from 'vue'

import { api } from '../helpers/api'
import { IMPORT_SOURCE_NAMES, importSourceName } from '../helpers/sourceNames'
import type { ImportCandidate, ScannableSource, ScanProgressEvent } from '../types'
import { useAppState } from './useAppState'
import { useTaskStatus } from './useTaskStatus'

export interface SourceState {
	key: ScannableSource
	name: string
	status: 'pending' | 'scanning' | 'done'
	found: number
}

export const SCANNABLE_SOURCES = Object.keys(IMPORT_SOURCE_NAMES).filter(
	(key): key is ScannableSource => key !== 'manual',
)

const sourceStates = ref<SourceState[]>(makeSourceStates(SCANNABLE_SOURCES))
let unlistenScan: (() => void) | undefined

function makeSourceStates(sources: ScannableSource[]): SourceState[] {
	return sources.map((key) => ({
		key,
		name: importSourceName(key),
		status: 'pending' as const,
		found: 0,
	}))
}

function mergeCandidates(existing: ImportCandidate[], incoming: ImportCandidate[]) {
	const map = new Map(existing.map((c) => [c.id, c]))
	for (const candidate of incoming) {
		map.set(candidate.id, candidate)
	}
	return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name))
}

export function useScanSources() {
	const state = useAppState()
	const task = useTaskStatus()

	async function scan() {
		if (!state.selectedUserId.value || !state.settingsReady.value) return

		const enabledSources = SCANNABLE_SOURCES.filter((key) => state.settings.sources[key].enabled)
		sourceStates.value = makeSourceStates(enabledSources)
		state.scanPhase.value = 'scanning'

		unlistenScan?.()
		unlistenScan = await listen<ScanProgressEvent>('scan-progress', (event) => {
			const { source, status, found } = event.payload
			const entry =
				typeof source === 'string' ? sourceStates.value.find((s) => s.key === source) : undefined
			if (entry) {
				if (status === 'scanning') {
					entry.status = 'scanning'
				} else if (status === 'done') {
					entry.status = 'done'
					entry.found = found
				}
			}
		})

		const found = await task.runTask('Scanning sources', () =>
			api.scanSources({ userSteamId: state.selectedUserId.value, includeSources: [] }),
		)

		unlistenScan()
		unlistenScan = undefined

		if (found !== undefined) {
			state.candidates.value = mergeCandidates(state.candidates.value, found)
			state.selectedCandidateIds.value = new Set(state.candidates.value.map((c) => c.id))
			state.invalidatePreview()
			state.scanPhase.value = 'done'
		} else {
			state.scanPhase.value = 'idle'
		}
	}

	return { sourceStates, scan }
}
