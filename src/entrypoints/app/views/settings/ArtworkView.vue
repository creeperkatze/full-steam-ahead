<script setup lang="ts">
import { ExternalLink, Eye, Images, KeyRound } from '@lucide/vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import OptionToggle from '../../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import UiButton from '../../../../components/ui/Button.vue'
import { useAppState } from '../../../../composables/useAppState'

const state = useAppState()
const { t } = useI18n()

const apiKey = computed({
	get: () => state.settings.steamGridDb.apiKey ?? '',
	set: (value: string) => {
		state.settings.steamGridDb.apiKey = value.trim() || null
	},
})
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader :title="t('settings.artwork.title')" />
		<div class="flex flex-col gap-2">
			<OptionToggle
				v-model="state.settings.steamGridDb.enabled"
				:icon="Images"
				:label="t('settings.artwork.enableSteamGridDb.label')"
				:description="t('settings.artwork.enableSteamGridDb.description')"
			/>
			<div
				class="flex min-w-0 flex-col gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2"
			>
				<div class="flex min-w-0 items-center gap-3">
					<KeyRound :size="18" class="shrink-0 text-secondary" />
					<div class="min-w-0 flex-1">
						<p class="text-sm font-medium">{{ t('settings.artwork.apiKey.label') }}</p>
						<p class="mt-0.5 text-xs text-secondary">
							{{ t('settings.artwork.apiKey.description') }}
						</p>
					</div>
				</div>
				<div class="flex items-center gap-2">
					<input
						v-model="apiKey"
						type="password"
						:placeholder="t('settings.artwork.apiKey.placeholder')"
						class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-4 px-2 text-sm text-primary"
					/>
					<UiButton
						size="icon"
						variant="ghost"
						:title="t('settings.artwork.apiKey.getKeyTitle')"
						@click="openUrl('https://www.steamgriddb.com/profile/preferences/api')"
					>
						<ExternalLink :size="16" />
					</UiButton>
				</div>
			</div>
			<OptionToggle
				v-model="state.settings.steamGridDb.allowNsfw"
				:icon="Eye"
				:label="t('settings.artwork.allowNsfw.label')"
				:description="t('settings.artwork.allowNsfw.description')"
			/>
		</div>
	</section>
</template>
