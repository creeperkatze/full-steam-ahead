import type { Stat } from '../shared/theme'

const REPO = 'creeperkatze/full-steam-ahead'

interface GitHubRelease {
	assets: { name: string; download_count: number }[]
}

const PLATFORMS = [
	{ label: 'Windows', marker: '-windows-' },
	{ label: 'macOS', marker: '-darwin-' },
	{ label: 'Linux', marker: '-linux-' },
]

export async function loadDownloads(): Promise<Stat[]> {
	const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100`)
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

	return PLATFORMS.flatMap(({ label }, index) =>
		totals[index]
			? [{ label: (t) => t('stats.downloads', { platform: label }), value: totals[index] }]
			: [],
	)
}
