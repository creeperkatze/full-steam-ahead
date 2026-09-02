<script setup lang="ts">
import { Gamepad2, HardDrive, Layers, Power, RotateCw } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import OptionPath from '../../../../components/options/OptionPath.vue'
import OptionToggle from '../../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { useAppState } from '../../../../composables/useAppState'
import { api } from '../../../../helpers/api'

const state = useAppState()
const { t } = useI18n()

const steamLocation = computed({
	get: () => state.settings.steamLocation ?? '',
	set: (value: string) => {
		state.settings.steamLocation = value.trim() || null
	},
})

async function pickSteamLocation() {
	const picked = await open({ directory: true, multiple: false })
	if (typeof picked === 'string') {
		steamLocation.value = picked
	}
}

const steamLocationValid = ref<boolean | null>(null)
let steamLocationCheckTimer: ReturnType<typeof setTimeout> | undefined

watch(
	() => state.settings.steamLocation,
	(path) => {
		clearTimeout(steamLocationCheckTimer)
		const trimmed = (path ?? '').trim()
		if (!trimmed) {
			steamLocationValid.value = null
			return
		}
		steamLocationCheckTimer = setTimeout(async () => {
			const valid = await api.validateSteamLocation(trimmed)
			if ((state.settings.steamLocation ?? '').trim() === trimmed) {
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
		<SectionHeader :title="t('settings.steam.title')" />
		<div class="flex flex-col gap-2">
			<OptionToggle
				v-model="state.settings.stopSteam"
				:icon="Power"
				:label="t('settings.steam.stopSteam.label')"
				:description="t('settings.steam.stopSteam.description')"
			/>
			<OptionToggle
				v-model="state.settings.restartSteam"
				:icon="RotateCw"
				:label="t('settings.steam.restartSteam.label')"
				:description="t('settings.steam.restartSteam.description')"
			/>
			<OptionToggle
				v-model="state.settings.createCollections"
				:icon="Layers"
				:label="t('settings.steam.createCollections.label')"
				:description="t('settings.steam.createCollections.description')"
			/>
			<OptionToggle
				v-model="state.settings.addSelfShortcut"
				:icon="Gamepad2"
				:label="t('settings.steam.addSelfShortcut.label')"
				:description="t('settings.steam.addSelfShortcut.description')"
			/>
			<OptionPath
				v-model="steamLocation"
				:icon="HardDrive"
				:label="t('settings.steam.steamLocation.label')"
				:description="t('settings.steam.steamLocation.description')"
				:placeholder="t('common.autoDetected')"
				:valid="steamLocationValid"
				@browse="pickSteamLocation"
			/>
		</div>
	</section>
</template>
