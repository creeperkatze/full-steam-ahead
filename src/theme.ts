export type ColorScheme = 'light' | 'dark'

export function applyColorScheme(scheme: ColorScheme | null) {
	if (scheme) {
		document.documentElement.dataset.theme = scheme
	} else {
		delete document.documentElement.dataset.theme
	}
}
