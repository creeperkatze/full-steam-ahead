<script setup lang="ts">
import { AlertCircle, FolderOpen } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import OptionButton from '../../../../components/options/OptionButton.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import { api } from '../../../../helpers/api'
import type { DebugInfo } from '../../../../types'

const { t } = useI18n()

const debugInfo = ref<DebugInfo | null>(null)
const loading = ref(true)
const openError = ref<string | null>(null)

const metadataFields = computed(() => {
	if (!debugInfo.value) return []
	return [
		{ label: t('settings.debug.fields.version'), value: debugInfo.value.appVersion },
		{
			label: t('settings.debug.fields.platform'),
			value: `${debugInfo.value.os} (${debugInfo.value.arch})`,
		},
		{ label: t('settings.debug.fields.dataPath'), value: debugInfo.value.dataPath },
	]
})

onMounted(async () => {
	try {
		debugInfo.value = await api.getDebugInfo()
	} finally {
		loading.value = false
	}
})

async function openLogsFolder() {
	openError.value = null
	try {
		await api.openLogsFolder()
	} catch (e: unknown) {
		openError.value = (e as { message?: string })?.message ?? t('settings.debug.openError')
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader :title="t('settings.debug.title')" />
		<div class="flex flex-col gap-2">
			<div class="rounded-lg border border-border bg-surface-3 px-3 py-2.5">
				<p v-if="loading" class="text-sm text-secondary">
					{{ t('settings.debug.loadingMetadata') }}
				</p>
				<dl
					v-else-if="debugInfo"
					class="grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-2 text-sm"
				>
					<template v-for="field in metadataFields" :key="field.label">
						<dt class="text-secondary">{{ field.label }}</dt>
						<dd class="min-w-0">
							<code
								class="inline-block max-w-full rounded bg-surface-4 px-1.5 py-0.5 font-mono text-xs break-all"
								>{{ field.value }}</code
							>
						</dd>
					</template>
				</dl>
			</div>

			<OptionButton
				:icon="FolderOpen"
				:label="t('settings.debug.logsFolder.label')"
				:description="t('settings.debug.logsFolder.description')"
				:button-label="t('settings.debug.logsFolder.open')"
				@click="openLogsFolder"
			/>

			<div v-if="openError" class="flex items-center gap-2 text-sm text-danger">
				<AlertCircle :size="15" class="shrink-0" />
				{{ openError }}
			</div>
		</div>
	</section>
</template>
