<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed } from 'vue'

import OptionLauncher from '../../../../components/options/OptionLauncher.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { importSourceName } from '../../../../helpers/sourceNames'
import type { ImportSource } from '../../../../types'

const state = useAppState()

const PATHLESS_LAUNCHERS = new Set(['gamePass'])

const launcherRows = computed(() =>
	// `availableLaunchers` only ever contains scannable launcher keys (plain strings), never `manual` or `{ other }`.
	(state.availableLaunchers.value as string[])
		.map((key) => ({
			key,
			label: importSourceName(key as ImportSource),
			pathSupported: !PATHLESS_LAUNCHERS.has(key),
		}))
		.sort((a, b) => a.label.localeCompare(b.label)),
)

function launcherSettingsFor(key: string) {
	if (!state.launcherSettings.value[key]) {
		state.launcherSettings.value[key] = { enabled: true, customPath: '' }
	}
	return state.launcherSettings.value[key]
}

async function pickLauncherPath(key: string) {
	const picked = await open({ directory: true, multiple: false })
	if (typeof picked === 'string') {
		launcherSettingsFor(key).customPath = picked
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader title="Launchers" />
		<p class="mb-3 text-xs text-secondary">
			Disable launchers you don't want scanned, or point at a non-standard install location.
		</p>
		<div class="grid gap-2">
			<OptionLauncher
				v-for="launcher in launcherRows"
				:key="launcher.key"
				:label="launcher.label"
				:enabled="launcherSettingsFor(launcher.key).enabled"
				:custom-path="launcherSettingsFor(launcher.key).customPath"
				:path-supported="launcher.pathSupported"
				@update:enabled="launcherSettingsFor(launcher.key).enabled = $event"
				@update:custom-path="launcherSettingsFor(launcher.key).customPath = $event"
				@browse="pickLauncherPath(launcher.key)"
			/>
		</div>
	</section>
</template>
