import { createI18n } from 'vue-i18n'

import de from './locales/de.json'
import en from './locales/en.json'

const messages = { en, de }

export interface LocaleDefinition {
	code: keyof typeof messages
	name: string
}

export const LOCALES: LocaleDefinition[] = [
	{ code: 'en', name: 'English' },
	{ code: 'de', name: 'Deutsch' },
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
