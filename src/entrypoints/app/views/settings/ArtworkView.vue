<script setup lang="ts">
import { ExternalLink, Eye, Images, KeyRound } from '@lucide/vue'
import { openUrl } from '@tauri-apps/plugin-opener'

import OptionToggle from '../../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import UiButton from '../../../../components/ui/Button.vue'
import { useAppState } from '../../../../composables/useAppState'

const state = useAppState()
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader title="Artwork" />
		<div class="flex flex-col gap-2">
			<OptionToggle
				v-model="state.steamGridDb.value.enabled"
				:icon="Images"
				label="Enable SteamGridDB"
				description="Adds a button to pick custom artwork from SteamGridDB when reviewing games"
			/>
			<OptionToggle
				v-model="state.steamGridDb.value.allowNsfw"
				:icon="Eye"
				label="Allow NSFW artwork"
				description="Includes NSFW-tagged images in SteamGridDB results"
			/>
			<div
				class="flex min-w-0 flex-col gap-2 rounded-lg border border-border bg-surface-3 px-3 py-2"
			>
				<div class="flex min-w-0 items-center gap-3">
					<KeyRound :size="18" class="shrink-0 text-secondary" />
					<div class="min-w-0 flex-1">
						<p class="text-sm font-medium">SteamGridDB API key</p>
						<p class="mt-0.5 text-xs text-secondary">
							Required for SteamGridDB. Generate one for free on your account page.
						</p>
					</div>
				</div>
				<div class="flex items-center gap-2">
					<input
						v-model="state.steamGridDb.value.apiKey"
						type="password"
						placeholder="Paste your API key"
						class="h-9 min-w-0 flex-1 rounded-md border border-border bg-surface-4 px-2 text-sm text-primary"
					/>
					<UiButton
						size="icon"
						variant="ghost"
						title="Get an API key on steamgriddb.com"
						@click="openUrl('https://www.steamgriddb.com/profile/preferences/api')"
					>
						<ExternalLink :size="16" />
					</UiButton>
				</div>
			</div>
		</div>
	</section>
</template>
