<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { computed } from 'vue'

import OptionSource from '../../../../components/options/OptionSource.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { importSourceName } from '../../../../helpers/sourceNames'
import type { ImportSource } from '../../../../types'

const state = useAppState()

const PATHLESS_SOURCES = new Set(['gamePass'])

const sourceRows = computed(() =>
	(state.availableSources.value as string[])
		.map((key) => ({
			key,
			label: importSourceName(key as ImportSource),
			pathSupported: !PATHLESS_SOURCES.has(key),
		}))
		.sort((a, b) => a.label.localeCompare(b.label)),
)

function sourceSettingsFor(key: string) {
	if (!state.sourceSettings.value[key]) {
		state.sourceSettings.value[key] = { enabled: true, customPath: '' }
	}
	return state.sourceSettings.value[key]
}

async function pickSourcePath(key: string) {
	const picked = await open({ directory: true, multiple: false })
	if (typeof picked === 'string') {
		sourceSettingsFor(key).customPath = picked
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader title="Sources" />
		<div class="grid gap-2">
			<OptionSource
				v-for="source in sourceRows"
				:key="source.key"
				:source="source.key"
				:label="source.label"
				:enabled="sourceSettingsFor(source.key).enabled"
				:custom-path="sourceSettingsFor(source.key).customPath"
				:path-supported="source.pathSupported"
				@update:enabled="sourceSettingsFor(source.key).enabled = $event"
				@update:custom-path="sourceSettingsFor(source.key).customPath = $event"
				@browse="pickSourcePath(source.key)"
			/>
		</div>
	</section>
</template>
