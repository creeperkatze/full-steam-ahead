import { createI18n } from 'vue-i18n'

import de from './locales/de.json'
import en from './locales/en.json'

const messages = { en, de }

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
	{ code: 'en', name: 'English' },
	// { code: 'fi-FI', name: 'Suomi' },
	// { code: 'fr-FR', name: 'Français' },
	{ code: 'de', name: 'Deutsch' },
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
	for (const lang of navigator.languages?.length ? navigator.languages : [navigator.language]) {
		const prefix = lang.split('-')[0].toLowerCase()
		if (isSupportedLocale(prefix)) return prefix
	}
	return 'en'
}

export const i18n = createI18n<[(typeof messages)['en']], SupportedLocale, false>({
	legacy: false,
	locale: detectBrowserLocale(),
	fallbackLocale: 'en',
	messages,
})

export function applyLocale(locale: SupportedLocale | null) {
	i18n.global.locale.value = locale ?? detectBrowserLocale()
}
