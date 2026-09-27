<script setup lang="ts">
import { Loader2, ShieldAlert } from '@lucide/vue'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { commands } from '../bindings'
import { useAppState } from '../composables/useAppState'
import Modal from './Modal.vue'
import UiButton from './ui/Button.vue'

defineProps<{ modelValue: boolean }>()

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const { t } = useI18n()
const state = useAppState()

type Status = 'prompt' | 'granting' | 'granted' | 'error'
const status = ref<Status>('prompt')
const errorMessage = ref('')

async function grant() {
	status.value = 'granting'
	try {
		await commands.grantSteamFlatpakPermission()
		status.value = 'granted'
		state.install.value = await commands.detectSteam()
	} catch (error) {
		errorMessage.value = error instanceof Error ? error.message : String(error)
		status.value = 'error'
	}
}

function close() {
	emit('update:modelValue', false)
}
</script>

<template>
	<Modal :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)">
		<div class="mb-5 flex items-start gap-3">
			<ShieldAlert :size="20" class="mt-0.5 shrink-0 text-accent" />
			<div>
				<h2 class="mb-1.5 text-base font-semibold">{{ t('flatpakPermissionModal.title') }}</h2>
				<p class="text-sm text-secondary">
					<template v-if="status === 'granted'">
						{{ t('flatpakPermissionModal.granted') }}
					</template>
					<template v-else-if="status === 'error'">
						{{ t('flatpakPermissionModal.error', { message: errorMessage }) }}
					</template>
					<template v-else>
						{{ t('flatpakPermissionModal.description') }}
					</template>
				</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton v-if="status === 'granted'" variant="primary" @click="close">
				{{ t('flatpakPermissionModal.close') }}
			</UiButton>
			<template v-else>
				<UiButton variant="ghost" @click="close">
					{{ t('flatpakPermissionModal.later') }}
				</UiButton>
				<UiButton variant="primary" :disabled="status === 'granting'" @click="grant">
					<Loader2 v-if="status === 'granting'" :size="14" class="animate-spin" />
					{{ t('flatpakPermissionModal.grant') }}
				</UiButton>
			</template>
		</div>
	</Modal>
</template>
