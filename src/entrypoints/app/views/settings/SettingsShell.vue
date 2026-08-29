<script setup lang="ts">
import { Archive, Gamepad2, Images, Rocket } from '@lucide/vue'
import { useRoute, useRouter } from 'vue-router'

import GitHubIcon from '../../../../assets/icons/github.svg?component'
import KofiIcon from '../../../../assets/icons/kofi.svg?component'
import Card from '../../../../components/Card.vue'
import SidebarTab from '../../../../components/options/SidebarTab.vue'

const route = useRoute()
const router = useRouter()

const sections = [
	{ name: 'settings-steam', label: 'Steam', icon: Gamepad2 },
	{ name: 'settings-launchers', label: 'Launchers', icon: Rocket },
	{ name: 'settings-artwork', label: 'Artwork', icon: Images },
	{ name: 'settings-backups', label: 'Backups', icon: Archive },
] as const
</script>

<template>
	<div class="flex h-full min-h-0 flex-1">
		<nav class="flex w-48 shrink-0 flex-col gap-1 pr-4">
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
					title="Donate on Ko-fi"
					description="Buy me a coffee"
				/>
				<Card
					href="https://github.com/creeperkatze/full-steam-ahead"
					:icon="GitHubIcon"
					color="#9ca3af"
					title="View on GitHub"
					description="Leave a star"
				/>
			</div>
		</nav>

		<div class="min-h-0 min-w-0 flex-1 overflow-y-auto border-l border-border pl-4">
			<RouterView />
		</div>
	</div>
</template>
