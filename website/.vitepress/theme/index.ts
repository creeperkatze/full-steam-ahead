/* eslint-disable simple-import-sort/imports */

import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'

import DonateButton from './DonateButton.vue'
import HeroLogo from './HeroLogo.vue'
import SiteFooter from './SiteFooter.vue'
import './custom.css'

export default {
	extends: DefaultTheme,
	Layout() {
		return h(DefaultTheme.Layout, null, {
			'nav-bar-content-after': () => h(DonateButton),
			'home-hero-info-before': () => h(HeroLogo),
			'layout-bottom': () => h(SiteFooter),
		})
	},
}
