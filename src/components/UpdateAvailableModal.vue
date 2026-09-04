<script setup lang="ts">
import { Clock } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import Modal from './Modal.vue'
import UiButton from './ui/Button.vue'

defineProps<{
	modelValue: boolean
	currentVersion: string
	latestVersion: string
}>()

const emit = defineEmits<{
	'update:modelValue': [value: boolean]
	download: []
}>()

const { t } = useI18n()
</script>

<template>
	<Modal :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)">
		<div class="mb-5 flex items-start gap-3">
			<Clock :size="20" class="mt-0.5 shrink-0 text-accent" />
			<div>
				<h2 class="mb-1.5 text-sm font-semibold">{{ t('updateModal.title') }}</h2>
				<p class="text-xs text-secondary">
					{{ t('updateModal.description', { current: currentVersion, latest: latestVersion }) }}
				</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton variant="ghost" @click="emit('update:modelValue', false)">
				{{ t('updateModal.later') }}
			</UiButton>
			<UiButton variant="primary" @click="emit('download')">
				{{ t('updateModal.download') }}
			</UiButton>
		</div>
	</Modal>
</template>
