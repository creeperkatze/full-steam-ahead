import type { ArtworkAsset, ArtworkKind, ImportCandidate } from '../types'

// Must match SELECTION_ORDER in src-tauri/src/steam/artwork/mod.rs.
const SELECTION_ORDER: ArtworkAsset['source'][] = [
	'missing',
	'localFile',
	'steamGridDb',
	'officialSteam',
]

// Mirrors selected_artwork_assets in src-tauri/src/steam/artwork/mod.rs.
export function displayAsset(
	candidate: ImportCandidate,
	kind: ArtworkKind,
): ArtworkAsset | undefined {
	const matches = candidate.artwork.proposed.filter((asset) => asset.kind === kind)
	const proposed = (chosen: boolean) =>
		SELECTION_ORDER.map((source) =>
			matches.find((asset) => asset.source === source && asset.chosen === chosen),
		).find((asset) => asset !== undefined)

	const existing = candidate.artwork.existing.find((asset) => asset.kind === kind)
	return (
		proposed(true) ?? (candidate.existingAppId !== null && existing ? existing : proposed(false))
	)
}
