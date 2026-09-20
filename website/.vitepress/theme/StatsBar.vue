<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

const REPO = 'creeperkatze/full-steam-ahead'

interface GitHubAsset {
	name: string
	download_count: number
}

interface GitHubRelease {
	assets: GitHubAsset[]
}

interface Downloads {
	windows: number
	macos: number
	linux: number
}

const downloads = ref<Downloads | null>(null)

onMounted(async () => {
	try {
		const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=100`)
		if (!res.ok) throw new Error(`releases request failed with status ${res.status}`)
		const releases = (await res.json()) as GitHubRelease[]

		const totals: Downloads = { windows: 0, macos: 0, linux: 0 }
		for (const release of releases) {
			for (const asset of release.assets) {
				// Skip auto-updater manifests/signatures, they're fetched repeatedly by
				// installed copies checking for updates, not by people downloading the app
				if (asset.name === 'latest.json' || asset.name.endsWith('.sig')) continue

				if (asset.name.includes('-windows-')) totals.windows += asset.download_count
				else if (asset.name.includes('-darwin-')) totals.macos += asset.download_count
				else if (asset.name.includes('-linux-')) totals.linux += asset.download_count
			}
		}
		downloads.value = totals
	} catch {
		// Download stats are a nice-to-have, fail silently rather than showing a broken widget
	}
})

const cards = computed(() => {
	if (!downloads.value) return []
	const { windows, macos, linux } = downloads.value

	const list: { label: string; value: string }[] = []
	if (windows) list.push({ label: 'Windows Downloads', value: windows.toLocaleString() })
	if (macos) list.push({ label: 'macOS Downloads', value: macos.toLocaleString() })
	if (linux) list.push({ label: 'Linux Downloads', value: linux.toLocaleString() })
	return list
})
</script>

<template>
	<div v-if="cards.length" class="stats-bar">
		<div class="stats-grid">
			<article v-for="card in cards" :key="card.label" class="stat-card">
				<span class="stat-value">{{ card.value }}</span>
				<span class="stat-label">{{ card.label }}</span>
			</article>
		</div>
	</div>
</template>

<style scoped>
.stats-bar {
	position: relative;
	padding: 0 24px 16px;
}

.stats-grid {
	display: flex;
	flex-wrap: wrap;
	gap: 16px;
	max-width: 1152px;
	margin: 0 auto;
}

.stat-card {
	flex: 1 1 200px;
	display: flex;
	flex-direction: column;
	align-items: center;
	text-align: center;
	padding: 24px;
	border-radius: 12px;
	border: 1px solid var(--vp-c-bg-soft);
	background-color: var(--vp-c-bg-soft);
}

.stat-value {
	font-size: 2.25rem;
	font-weight: 700;
	line-height: 1;
	background-image: linear-gradient(120deg, var(--vp-c-brand-1) 30%, var(--vp-c-brand-2));
	-webkit-background-clip: text;
	background-clip: text;
	color: transparent;
}

.stat-label {
	margin-top: 4px;
	font-size: 0.875rem;
	font-weight: 500;
	color: var(--vp-c-text-2);
}

@media (min-width: 640px) {
	.stats-bar {
		padding: 0 48px 16px;
	}

	.stat-card {
		flex-basis: calc(33.333% - 11px);
	}
}

@media (min-width: 960px) {
	.stats-bar {
		padding: 0 64px 16px;
	}
}
</style>
