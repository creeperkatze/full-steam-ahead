<script setup lang="ts">
import { Languages } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import OptionSelect from '../../../../components/options/OptionSelect.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { detectBrowserLocale, LOCALES, type SupportedLocale } from '../../../../i18n'

const state = useAppState()
const { t } = useI18n()

const languageOptions = computed(() => LOCALES.map((l) => ({ value: l.code, label: l.name })))
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
				:description="t('settings.customization.language.description')"
				:options="languageOptions"
				@update:model-value="state.locale.value = $event as SupportedLocale"
			/>
		</div>
	</section>
</template>
