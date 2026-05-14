export interface SteamInstallation {
	installPath: string
	users: SteamUser[]
	running: boolean
}

export interface SteamUser {
	steamId: string
	accountName?: string | null
	shortcutsPath: string
	gridPath: string
	collectionsPath: string
}

export interface ShortcutEntry {
	appId: number
	appName: string
	exe: string
	startDir: string
	icon: string
	shortcutPath: string
	launchOptions: string
	isHidden: boolean
	allowDesktopConfig: boolean
	allowOverlay: boolean
	openVr: boolean
	devkit: boolean
	devkitGameId: string
	lastPlayTime: number
	tags: string[]
}
