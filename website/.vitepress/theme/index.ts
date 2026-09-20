import './custom.css'

import DefaultTheme from 'vitepress/theme'
import { h } from 'vue'

import DonateButton from './DonateButton.vue'
import HeroActions from './HeroActions.vue'
import HeroLogo from './HeroLogo.vue'
import Showcase from './Showcase.vue'
import SiteFooter from './SiteFooter.vue'
import StatsBar from './StatsBar.vue'

export default {
	extends: DefaultTheme,
	Layout() {
		return h(DefaultTheme.Layout, null, {
			'nav-bar-content-after': () => h(DonateButton),
			'home-hero-info-before': () => h(HeroLogo),
			'home-hero-actions-after': () => h(HeroActions),
			'home-features-before': () => h(StatsBar),
			'home-features-after': () => h(Showcase),
			'layout-bottom': () => h(SiteFooter),
		})
	},
}
