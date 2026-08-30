<script setup lang="ts">
import { Languages } from '@lucide/vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Component } from 'vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import DeFlag from '../../../../assets/icons/flags/de.svg?component'
import GbFlag from '../../../../assets/icons/flags/gb.svg?component'
import OptionSelect from '../../../../components/options/OptionSelect.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { detectBrowserLocale, LOCALES, type SupportedLocale } from '../../../../i18n'

const FLAGS: Record<SupportedLocale, Component> = {
	en: GbFlag,
	de: DeFlag,
}

const state = useAppState()
const { t } = useI18n()

const languageOptions = computed(() =>
	LOCALES.map((l) => ({ value: l.code, label: l.name, flag: FLAGS[l.code] })),
)
const selectedLocale = computed(() => state.locale.value ?? detectBrowserLocale())
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
				@update:model-value="state.locale.value = $event as SupportedLocale"
			>
				<template #leading="{ option }">
					<component :is="option.flag" class="h-3 w-4 shrink-0 rounded-[1px] object-cover" />
				</template>
				<template #description>
					<i18n-t keypath="settings.customization.language.description" tag="span">
						<template #crowdin>
							<button
								type="button"
								class="cursor-pointer text-accent hover:underline"
								@click.stop="openUrl('https://crowdin.com/project/full-steam-ahead')"
							>
								Crowdin
							</button>
						</template>
					</i18n-t>
				</template>
			</OptionSelect>
		</div>
	</section>
</template>
