<script setup lang="ts">
import { Archive, Bug, FolderInput, Gamepad2, Images, Palette } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import CrowdinIcon from '../../../../assets/icons/crowdin.svg?component'
import GitHubIcon from '../../../../assets/icons/github.svg?component'
import KofiIcon from '../../../../assets/icons/kofi.svg?component'
import Card from '../../../../components/Card.vue'
import SidebarTab from '../../../../components/options/SidebarTab.vue'
import { useAppState } from '../../../../composables/useAppState'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const state = useAppState()

const sections = computed(
	() =>
		[
			{
				name: 'settings-customization',
				label: t('settingsShell.tabs.customization'),
				icon: Palette,
			},
			{ name: 'settings-steam', label: t('settingsShell.tabs.steam'), icon: Gamepad2 },
			{ name: 'settings-sources', label: t('settingsShell.tabs.sources'), icon: FolderInput },
			{ name: 'settings-artwork', label: t('settingsShell.tabs.artwork'), icon: Images },
			{ name: 'settings-backups', label: t('settingsShell.tabs.backups'), icon: Archive },
			{ name: 'settings-debug', label: t('settingsShell.tabs.debug'), icon: Bug },
		] as const,
)
</script>

<template>
	<div class="flex h-full min-h-0 flex-1">
		<nav class="flex w-56 shrink-0 flex-col gap-1.5 pr-4">
			<SidebarTab
				v-for="section in sections"
				:key="section.name"
				:icon="section.icon"
				:label="section.label"
				:active="route.name === section.name"
				@click="router.push({ name: section.name })"
			/>

			<div class="mt-auto flex flex-col gap-1.5 pt-4">
				<Card
					href="https://ko-fi.com/creeperkatze"
					:icon="KofiIcon"
					color="#FF5E5B"
					:title="t('settingsShell.donateCard.title')"
					:description="t('settingsShell.donateCard.description')"
				/>
				<Card
					href="https://crowdin.com/project/full-steam-ahead"
					:icon="CrowdinIcon"
					color="#9ca3af"
					:title="t('settingsShell.crowdinCard.title')"
					:description="t('settingsShell.crowdinCard.description')"
				/>
				<Card
					href="https://github.com/creeperkatze/full-steam-ahead"
					:icon="GitHubIcon"
					color="#9ca3af"
					:title="t('settingsShell.githubCard.title')"
					:description="t('settingsShell.githubCard.description')"
				/>
			</div>
		</nav>

		<div class="min-h-0 min-w-0 flex-1 overflow-y-auto border-l border-border pl-4">
			<RouterView v-if="state.settingsReady.value" />
		</div>
	</div>
</template>
