<script setup lang="ts">
import { AlertCircle, AlertTriangle, CheckCircle2, Download, RotateCcw, Upload } from '@lucide/vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import Modal from '../../../../components/Modal.vue'
import OptionButton from '../../../../components/options/OptionButton.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import UiButton from '../../../../components/ui/Button.vue'
import { useAppState } from '../../../../composables/useAppState'
import { api } from '../../../../helpers/api'

const { t } = useI18n()
const state = useAppState()

const busy = ref(false)
const feedback = ref<{ type: 'success' | 'error'; message: string } | null>(null)
const showExportWarning = ref(false)
const showResetConfirm = ref(false)

function startExport() {
	feedback.value = null
	showExportWarning.value = true
}

async function confirmExport() {
	showExportWarning.value = false
	const path = await save({
		defaultPath: 'full-steam-ahead-settings.json',
		filters: [{ name: 'JSON', extensions: ['json'] }],
	})
	if (!path) return
	busy.value = true
	feedback.value = null
	try {
		await api.exportSettings(path, { ...state.settings })
		feedback.value = { type: 'success', message: t('settings.data.export.success') }
	} catch (e: unknown) {
		feedback.value = {
			type: 'error',
			message: (e as { message?: string })?.message ?? t('settings.data.export.error'),
		}
	} finally {
		busy.value = false
	}
}

async function triggerImport() {
	feedback.value = null
	const path = await open({
		multiple: false,
		filters: [{ name: 'JSON', extensions: ['json'] }],
	})
	if (typeof path !== 'string') return
	busy.value = true
	try {
		const imported = await api.importSettings(path)
		state.applySettings(imported)
		feedback.value = { type: 'success', message: t('settings.data.import.success') }
	} catch (e: unknown) {
		feedback.value = {
			type: 'error',
			message: (e as { message?: string })?.message ?? t('settings.data.import.error'),
		}
	} finally {
		busy.value = false
	}
}

function startReset() {
	feedback.value = null
	showResetConfirm.value = true
}

async function confirmReset() {
	showResetConfirm.value = false
	busy.value = true
	try {
		const defaults = await api.resetSettings()
		state.applySettings(defaults)
		feedback.value = { type: 'success', message: t('settings.data.reset.success') }
	} catch (e: unknown) {
		feedback.value = {
			type: 'error',
			message: (e as { message?: string })?.message ?? t('settings.data.reset.error'),
		}
	} finally {
		busy.value = false
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader :title="t('settings.data.title')" />
		<div class="flex flex-col gap-2">
			<OptionButton
				:icon="Download"
				:label="t('settings.data.export.label')"
				:description="t('settings.data.export.description')"
				:button-label="t('settings.data.export.buttonLabel')"
				:disabled="busy"
				@click="startExport"
			/>
			<OptionButton
				:icon="Upload"
				:label="t('settings.data.import.label')"
				:description="t('settings.data.import.description')"
				:button-label="t('settings.data.import.buttonLabel')"
				:disabled="busy"
				@click="triggerImport"
			/>
			<OptionButton
				:icon="RotateCcw"
				:label="t('settings.data.reset.label')"
				:description="t('settings.data.reset.description')"
				:button-label="t('settings.data.reset.buttonLabel')"
				:disabled="busy"
				@click="startReset"
			/>

			<div
				v-if="feedback"
				class="flex items-center gap-2 text-sm"
				:class="feedback.type === 'error' ? 'text-danger' : ''"
			>
				<CheckCircle2 v-if="feedback.type === 'success'" :size="15" class="shrink-0 text-accent" />
				<AlertCircle v-else :size="15" class="shrink-0" />
				{{ feedback.message }}
			</div>
		</div>
	</section>

	<Modal v-model="showExportWarning">
		<div class="mb-5 flex items-start gap-3">
			<AlertTriangle :size="20" class="mt-0.5 shrink-0 text-warning" />
			<div>
				<h2 class="mb-1.5 text-sm font-semibold">{{ t('settings.data.export.warning.title') }}</h2>
				<p class="text-xs text-secondary">{{ t('settings.data.export.warning.description') }}</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton variant="ghost" @click="showExportWarning = false">{{
				t('common.cancel')
			}}</UiButton>
			<UiButton variant="danger" @click="confirmExport">
				{{ t('settings.data.export.warning.confirm') }}
			</UiButton>
		</div>
	</Modal>

	<Modal v-model="showResetConfirm">
		<div class="mb-5 flex items-start gap-3">
			<AlertTriangle :size="20" class="mt-0.5 shrink-0 text-warning" />
			<div>
				<h2 class="mb-1.5 text-sm font-semibold">{{ t('settings.data.reset.confirm.title') }}</h2>
				<p class="text-xs text-secondary">{{ t('settings.data.reset.confirm.description') }}</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton variant="ghost" @click="showResetConfirm = false">{{
				t('common.cancel')
			}}</UiButton>
			<UiButton variant="danger" @click="confirmReset">
				{{ t('settings.data.reset.confirm.confirm') }}
			</UiButton>
		</div>
	</Modal>
</template>
