import { createI18n } from 'vue-i18n'

import deDE from './locales/de-DE.json'
import enUS from './locales/en-US.json'

const messages = { 'en-US': enUS, 'de-DE': deDE }

export interface LocaleDefinition {
	code: keyof typeof messages
	name: string
	dir?: 'ltr' | 'rtl'
}

export const LOCALES: LocaleDefinition[] = [
	// { code: 'af-ZA', name: 'Afrikaans' },
	// { code: 'ar-SA', name: 'العربية', dir: 'rtl' },
	// { code: 'ca-ES', name: 'Català' },
	// { code: 'zh-CN', name: '简体中文' },
	// { code: 'zh-TW', name: '繁體中文' },
	// { code: 'cs-CZ', name: 'Čeština' },
	// { code: 'da-DK', name: 'Dansk' },
	// { code: 'nl-NL', name: 'Nederlands' },
	{ code: 'en-US', name: 'English' },
	// { code: 'fi-FI', name: 'Suomi' },
	// { code: 'fr-FR', name: 'Français' },
	{ code: 'de-DE', name: 'Deutsch' },
	// { code: 'el-GR', name: 'Ελληνικά' },
	// { code: 'he-IL', name: 'עברית', dir: 'rtl' },
	// { code: 'hu-HU', name: 'Magyar' },
	// { code: 'it-IT', name: 'Italiano' },
	// { code: 'ja-JP', name: '日本語' },
	// { code: 'ko-KR', name: '한국어' },
	// { code: 'no-NO', name: 'Norsk' },
	// { code: 'pl-PL', name: 'Polski' },
	// { code: 'pt-PT', name: 'Português' },
	// { code: 'pt-BR', name: 'Português (Brasil)' },
	// { code: 'ro-RO', name: 'Română' },
	// { code: 'ru-RU', name: 'Русский' },
	// { code: 'sr-CS', name: 'Српски' },
	// { code: 'es-ES', name: 'Español' },
	// { code: 'sv-SE', name: 'Svenska' },
	// { code: 'tr-TR', name: 'Türkçe' },
	// { code: 'uk-UA', name: 'Українська' },
	// { code: 'vi-VN', name: 'Tiếng Việt' },
]

export type SupportedLocale = LocaleDefinition['code']

const LOCALE_CODES = new Set(LOCALES.map((l) => l.code))

function isSupportedLocale(value: string): value is SupportedLocale {
	return LOCALE_CODES.has(value as SupportedLocale)
}

export function detectBrowserLocale(): SupportedLocale {
	const langs = navigator.languages?.length ? navigator.languages : [navigator.language]
	for (const lang of langs) {
		if (isSupportedLocale(lang)) return lang
	}
	for (const lang of langs) {
		const prefix = lang.split('-')[0].toLowerCase()
		const match = LOCALES.find((l) => l.code.split('-')[0] === prefix)
		if (match) return match.code
	}
	return 'en-US'
}

export const i18n = createI18n<[(typeof messages)['en-US']], SupportedLocale, false>({
	legacy: false,
	locale: detectBrowserLocale(),
	fallbackLocale: 'en-US',
	messages,
})

function setLocale(locale: SupportedLocale) {
	i18n.global.locale.value = locale
	document.documentElement.lang = locale
	document.documentElement.dir = LOCALES.find((l) => l.code === locale)?.dir ?? 'ltr'
}

setLocale(i18n.global.locale.value)

export function applyLocale(locale: SupportedLocale | null) {
	setLocale(locale ?? detectBrowserLocale())
}
