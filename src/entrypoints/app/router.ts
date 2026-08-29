import { createRouter, createWebHashHistory } from 'vue-router'

import ImportFlowView from './views/ImportFlowView.vue'
import ArtworkSettingsView from './views/settings/ArtworkView.vue'
import BackupsView from './views/settings/BackupsView.vue'
import DebugView from './views/settings/DebugView.vue'
import SettingsShell from './views/settings/SettingsShell.vue'
import SourcesView from './views/settings/SourcesView.vue'
import SteamView from './views/settings/SteamView.vue'

export const router = createRouter({
	history: createWebHashHistory(),
	routes: [
		{ path: '/', name: 'import', component: ImportFlowView },
		{
			path: '/settings',
			name: 'settings',
			component: SettingsShell,
			redirect: '/settings/steam',
			children: [
				{ path: 'steam', name: 'settings-steam', component: SteamView },
				{ path: 'sources', name: 'settings-sources', component: SourcesView },
				{ path: 'artwork', name: 'settings-artwork', component: ArtworkSettingsView },
				{ path: 'backups', name: 'settings-backups', component: BackupsView },
				{ path: 'debug', name: 'settings-debug', component: DebugView },
			],
		},
	],
})
