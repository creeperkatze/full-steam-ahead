import svgLoader from 'vite-svg-loader'
import { defineConfig } from 'vitepress'

const version = process.env.VERSION

const title = 'Full Steam Ahead'
const description = 'A desktop app to import games from other launchers into Steam.'
const url = 'https://full-steam-ahead.creeperkatze.dev'
const image = `${url}/banner.png`

export default defineConfig({
	title,
	description,
	cleanUrls: true,
	head: [
		['link', { rel: 'icon', type: 'image/png', href: '/favicon.png' }],
		['meta', { property: 'og:type', content: 'website' }],
		['meta', { property: 'og:url', content: url }],
		['meta', { property: 'og:title', content: title }],
		['meta', { property: 'og:description', content: description }],
		['meta', { property: 'og:image', content: image }],
		['meta', { name: 'twitter:card', content: 'summary_large_image' }],
		['meta', { name: 'twitter:title', content: title }],
		['meta', { name: 'twitter:description', content: description }],
		['meta', { name: 'twitter:image', content: image }],
	],
	vite: {
		plugins: [svgLoader()],
	},
	themeConfig: {
		logo: '/icon.svg',
		siteTitle: false,
		nav: [
			...(version
				? [
						{
							text: `v${version}`,
							link: 'https://github.com/creeperkatze/full-steam-ahead/releases',
						},
					]
				: []),
		],
		socialLinks: [{ icon: 'github', link: 'https://github.com/creeperkatze/full-steam-ahead' }],
	},
})
