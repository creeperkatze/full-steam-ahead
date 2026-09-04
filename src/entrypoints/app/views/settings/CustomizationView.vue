<script setup lang="ts">
import { Bell, Languages, Monitor } from '@lucide/vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Component } from 'vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import DeFlag from '../../../../assets/icons/flags/de.svg?component'
import GbFlag from '../../../../assets/icons/flags/gb.svg?component'
import OptionSelect from '../../../../components/options/OptionSelect.vue'
import OptionToggle from '../../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { detectBrowserLocale, LOCALES, type SupportedLocale } from '../../../../i18n'
import type { ColorScheme } from '../../../../theme'

const FLAGS: Record<SupportedLocale, Component> = {
	en: GbFlag,
	de: DeFlag,
}

const state = useAppState()
const { t } = useI18n()

const languageOptions = computed(() =>
	LOCALES.map((l) => ({ value: l.code, label: l.name, flag: FLAGS[l.code] })),
)
const selectedLocale = computed(() => state.settings.locale ?? detectBrowserLocale())

const colorSchemeOptions = computed(() => [
	{ value: 'auto', label: t('settings.customization.colorScheme.auto') },
	{ value: 'light', label: t('settings.customization.colorScheme.light') },
	{ value: 'dark', label: t('settings.customization.colorScheme.dark') },
])
const selectedColorScheme = computed(() => state.settings.colorScheme ?? 'auto')

function setColorScheme(value: string) {
	state.settings.colorScheme = value === 'auto' ? null : (value as ColorScheme)
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader :title="t('settings.customization.title')" />
		<div class="flex flex-col gap-2">
			<OptionSelect
				:model-value="selectedLocale"
				:icon="Languages"
				:label="t('settings.customization.language.label')"
				:options="languageOptions"
				@update:model-value="state.settings.locale = $event as SupportedLocale"
			>
				<template #leading="{ option }">
					<component :is="option.flag" class="h-3 w-4 shrink-0 rounded-[1px] object-cover" />
				</template>
				<template #description>
					<i18n-t keypath="settings.customization.language.description" tag="span">
						<template #crowdin>
							<button
								type="button"
								class="link"
								@click.stop="openUrl('https://crowdin.com/project/full-steam-ahead')"
							>
								Crowdin
							</button>
						</template>
					</i18n-t>
				</template>
			</OptionSelect>
			<OptionSelect
				:model-value="selectedColorScheme"
				:icon="Monitor"
				:label="t('settings.customization.colorScheme.label')"
				:description="t('settings.customization.colorScheme.description')"
				:options="colorSchemeOptions"
				@update:model-value="setColorScheme($event)"
			/>
			<OptionToggle
				v-model="state.settings.updateNotifications"
				:icon="Bell"
				:label="t('settings.customization.updateNotifications.label')"
				:description="t('settings.customization.updateNotifications.description')"
			/>
		</div>
	</section>
</template>
