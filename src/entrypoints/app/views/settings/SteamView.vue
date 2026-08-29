<script setup lang="ts">
import { HardDrive, Layers, Power, RotateCw } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { onUnmounted, ref, watch } from 'vue'

import OptionPath from '../../../../components/options/OptionPath.vue'
import OptionToggle from '../../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { api } from '../../../../helpers/api'

const state = useAppState()

async function pickSteamLocation() {
	const picked = await open({ directory: true, multiple: false })
	if (typeof picked === 'string') {
		state.steamLocation.value = picked
	}
}

const steamLocationValid = ref<boolean | null>(null)
let steamLocationCheckTimer: ReturnType<typeof setTimeout> | undefined

watch(
	() => state.steamLocation.value,
	(path) => {
		clearTimeout(steamLocationCheckTimer)
		const trimmed = path.trim()
		if (!trimmed) {
			steamLocationValid.value = null
			return
		}
		steamLocationCheckTimer = setTimeout(async () => {
			const valid = await api.validateSteamLocation(trimmed)
			if (state.steamLocation.value.trim() === trimmed) {
				steamLocationValid.value = valid
			}
		}, 400)
	},
	{ immediate: true },
)

onUnmounted(() => clearTimeout(steamLocationCheckTimer))
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader title="Steam" />
		<div class="flex flex-col gap-2">
			<OptionToggle
				v-model="state.options.value.stopSteam"
				:icon="Power"
				label="Stop Steam before applying"
				description="Steam must be closed to modify shortcut files"
			/>
			<OptionToggle
				v-model="state.options.value.restartSteam"
				:icon="RotateCw"
				label="Restart Steam after applying"
				description="Relaunches Steam so imported games appear immediately"
			/>
			<OptionToggle
				v-model="state.options.value.createCollections"
				:icon="Layers"
				label="Create per-platform collections"
				description="Groups imported games into a collection for each launcher"
			/>
			<OptionPath
				v-model="state.steamLocation.value"
				:icon="HardDrive"
				label="Steam installation location"
				description="Override auto-detection if Steam isn't found automatically"
				placeholder="Auto-detected"
				:valid="steamLocationValid"
				@browse="pickSteamLocation"
			/>
		</div>
	</section>
</template>
