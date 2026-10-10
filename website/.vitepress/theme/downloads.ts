import type { Stat } from '../shared/theme'

const REPO = 'creeperkatze/full-steam-ahead'
const FLATHUB_ID = 'dev.creeperkatze.full-steam-ahead'

interface GitHubRelease {
	assets: { name: string; download_count: number }[]
}

const PLATFORMS = [
	{ label: 'Windows', marker: '-windows-' },
	{ label: 'macOS', marker: '-darwin-' },
	{ label: 'Linux', marker: '-linux-' },
]

// Flathub's stats API has no CORS headers, so go through shields.io
async function loadFlathubInstalls(): Promise<number | null> {
	try {
		const res = await fetch(`https://img.shields.io/flathub/downloads/${FLATHUB_ID}.json`)
		if (!res.ok) return null
		const value = Number(((await res.json()) as { value?: string }).value)
		return Number.isFinite(value) ? value : null
	} catch {
		return null
	}
}

export async function loadDownloads(): Promise<Stat[]> {
	const [res, flathubInstalls] = await Promise.all([
		fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100`),
		loadFlathubInstalls(),
	])
	if (!res.ok) throw new Error(`releases request failed with status ${res.status}`)
	const releases = (await res.json()) as GitHubRelease[]

	const totals = PLATFORMS.map(() => 0)
	for (const release of releases) {
		for (const asset of release.assets) {
			// Skip auto-updater manifests/signatures, they're fetched repeatedly by
			// installed copies checking for updates, not by people downloading the app
			if (asset.name === 'latest.json' || asset.name.endsWith('.sig')) continue
			const index = PLATFORMS.findIndex(({ marker }) => asset.name.includes(marker))
			if (index !== -1) totals[index] += asset.download_count
		}
	}

	const stats: Stat[] = PLATFORMS.flatMap(({ label }, index) =>
		totals[index]
			? [{ label: (t) => t('stats.downloads', { platform: label }), value: totals[index] }]
			: [],
	)
	if (flathubInstalls) {
		stats.push({ label: (t) => t('stats.flathubInstalls'), value: flathubInstalls })
	}
	return stats
}
