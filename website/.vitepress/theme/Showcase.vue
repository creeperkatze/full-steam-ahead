<script setup lang="ts">
interface ShowcaseItem {
	title: string
	details: string
	image: string
}

const items: ShowcaseItem[] = [
	{
		title: 'Pick your Steam user',
		details:
			'Scan your installed launchers and pick which Steam user to import into, right when you open the app.',
		image: '/screenshots/start.png',
	},
	{
		title: 'Choose which games to import',
		details:
			'Every detected game from Epic, GOG, itch.io, and more is listed with a toggle, or add any executable manually as a custom non-Steam game.',
		image: '/screenshots/sources.png',
	},
	{
		title: 'Review matched grid images, heroes, and logos',
		details:
			'Full Steam Ahead fetches matching grid images, hero art, logos, and icons for every game it imports.',
		image: '/screenshots/artwork.png',
	},
	{
		title: 'Browse custom artwork on SteamGridDB',
		details:
			"Browse SteamGridDB's full library for every game and swap in the artwork you actually want.",
		image: '/screenshots/steamgriddb.png',
	},
	{
		title: 'Review before you apply anything',
		details:
			"See every Steam entry and launcher link that's about to be added or updated before you apply anything.",
		image: '/screenshots/review.png',
	},
	{
		title: 'All done, with an automatic backup',
		details:
			'Changes are applied and a backup of your Steam config is created automatically, so you can always roll back.',
		image: '/screenshots/done.png',
	},
	{
		title: 'Make it feel like yours',
		details:
			'Choose your language and switch between light, dark, or system color scheme, right from the settings.',
		image: '/screenshots/settings_customization.png',
	},
	{
		title: 'Decide how Steam gets handled',
		details:
			"Decide whether Steam is closed and restarted automatically, group imports into per-launcher collections, and override the install location if it isn't auto-detected.",
		image: '/screenshots/settings_steam.png',
	},
	{
		title: 'Configure sources',
		details:
			"Enable or disable each launcher, and override its auto-detected install path if Full Steam Ahead can't find it on its own.",
		image: '/screenshots/settings_sources.png',
	},
	{
		title: 'Choose where your artwork comes from',
		details:
			'Choose which source artwork is fetched from automatically, Steam or SteamGridDB, then enable SteamGridDB browsing and add your own API key.',
		image: '/screenshots/settings_artwork.png',
	},
	{
		title: 'Every backup, one restore away',
		details:
			"Browse every automatic backup Full Steam Ahead has made, restore any of them, or clean up the ones you don't need.",
		image: '/screenshots/settings_backups.png',
	},
]
</script>

<template>
	<div class="showcase">
		<div class="container">
			<div class="showcase-inner">
				<article
					v-for="(item, index) in items"
					:key="item.title"
					class="showcase-row"
					:class="{ reverse: index % 2 === 1 }"
				>
					<div class="showcase-media">
						<img
							:src="item.image"
							:alt="item.title"
							width="1202"
							height="712"
							loading="lazy"
							decoding="async"
						/>
					</div>
					<div class="showcase-text">
						<h3>{{ item.title }}</h3>
						<p>{{ item.details }}</p>
					</div>
				</article>
			</div>
		</div>
	</div>
</template>

<style scoped>
.showcase {
	position: relative;
	padding: 32px 24px 8px;
}

.container {
	max-width: 1152px;
	margin: 0 auto;
}

.showcase-inner {
	display: flex;
	flex-direction: column;
	gap: 64px;
}

.showcase-row {
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 24px;
}

.showcase-media {
	flex: 1 1 0;
	width: 100%;
}

.showcase-media img {
	width: 100%;
	aspect-ratio: 1202 / 712;
	object-fit: cover;
	border-radius: 12px;
	background-color: var(--vp-c-bg-soft);
}

.showcase-text {
	flex: 1 1 0;
	width: 100%;
}

.showcase-text h3 {
	font-size: 1.25rem;
	font-weight: 600;
	line-height: 1.4;
	margin: 0 0 8px;
	letter-spacing: -0.01em;
}

.showcase-text p {
	font-size: 0.95rem;
	line-height: 1.6;
	color: var(--vp-c-text-2);
	margin: 0;
}

@media (min-width: 640px) {
	.showcase {
		padding-left: 48px;
		padding-right: 48px;
	}
}

@media (min-width: 768px) {
	.showcase-row {
		flex-direction: row;
		align-items: center;
		gap: 48px;
	}

	.showcase-row.reverse {
		flex-direction: row-reverse;
	}
}

@media (min-width: 960px) {
	.showcase {
		padding-left: 64px;
		padding-right: 64px;
	}
}
</style>
