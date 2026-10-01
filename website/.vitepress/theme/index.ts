/* eslint-disable simple-import-sort/imports */

import { h } from 'vue'

import { createTheme, messagesFromGlob } from '../shared/theme'

import { loadDownloads } from './downloads'
import HeroActions from './HeroActions.vue'
import Logo from './icons/logo.svg?skipsvgo'
// Must come after the theme so the brand colors win
import './custom.css'

export default createTheme({
	messages: messagesFromGlob(
		import.meta.glob('../../src/locales/*.json', { eager: true, import: 'default' }),
	),
	logo: Logo,
	stats: loadDownloads,
	showcase: [
		{ key: 'steamUser', image: '/screenshots/start.png' },
		{ key: 'games', image: '/screenshots/sources.png' },
		{ key: 'artwork', image: '/screenshots/artwork.png' },
		{ key: 'steamGridDb', image: '/screenshots/steamgriddb.png' },
		{ key: 'review', image: '/screenshots/review.png' },
		{ key: 'done', image: '/screenshots/done.png' },
		{ key: 'customization', image: '/screenshots/settings_customization.png' },
		{ key: 'steamSettings', image: '/screenshots/settings_steam.png' },
		{ key: 'sources', image: '/screenshots/settings_sources.png' },
		{ key: 'artworkSource', image: '/screenshots/settings_artwork.png' },
		{ key: 'backups', image: '/screenshots/settings_backups.png' },
	],
	showcaseImageSize: { width: 1202, height: 712 },
	slots: {
		'home-hero-actions-after': () => h(HeroActions),
	},
})
