export interface Settings {
	stopSteam: boolean
	restartSteam: boolean
	createCollections: boolean
	steamLocation: string | null
	sources: Record<string, SourceSettings>
	steamGridDb: SteamGridDbSettings
	locale: string | null
	colorScheme: string | null
}

export interface SourceSettings {
	enabled: boolean
	customPath: string | null
}

export interface SteamGridDbSettings {
	enabled: boolean
	apiKey: string | null
	allowNsfw: boolean
}
