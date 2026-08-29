import { createRouter, createWebHashHistory } from 'vue-router'

import ImportFlowView from './views/ImportFlowView.vue'
import BackupsView from './views/settings/BackupsView.vue'
import LaunchersView from './views/settings/LaunchersView.vue'
import SettingsShell from './views/settings/SettingsShell.vue'
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
				{ path: 'launchers', name: 'settings-launchers', component: LaunchersView },
				{ path: 'backups', name: 'settings-backups', component: BackupsView },
			],
		},
	],
})
