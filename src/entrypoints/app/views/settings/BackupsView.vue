<script setup lang="ts">
import { AlertCircle, AlertTriangle, CheckCircle2, Loader2, RotateCcw, Trash2 } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import Modal from '../../../../components/Modal.vue'
import OptionButton from '../../../../components/options/OptionButton.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import UiButton from '../../../../components/ui/Button.vue'
import { api } from '../../../../helpers/api'
import type { BackupInfo } from '../../../../types'

const { t } = useI18n()

const backups = ref<BackupInfo[]>([])
const backupsLoading = ref(true)
const confirmingId = ref<string | null>(null)
const confirmingAction = ref<'restore' | 'delete' | 'delete-all' | null>(null)
const confirmingBackup = computed(() =>
	backups.value.find((backup) => backup.id === confirmingId.value),
)
const busy = ref(false)
const restoreResult = ref<{ backupId: string; count: number } | null>(null)
const actionError = ref<string | null>(null)

onMounted(async () => {
	try {
		backups.value = await api.listBackups()
	} catch {
		// Keep empty state
	} finally {
		backupsLoading.value = false
	}
})

function formatBackupDate(iso: string): string {
	return new Date(iso).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
}

function formatSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`
	if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`
	return `${(bytes / 1_048_576).toFixed(1)} MB`
}

function startRestore(backupId: string) {
	confirmingId.value = backupId
	confirmingAction.value = 'restore'
	restoreResult.value = null
	actionError.value = null
}

function startDelete(backupId: string) {
	confirmingId.value = backupId
	confirmingAction.value = 'delete'
	restoreResult.value = null
	actionError.value = null
}

function startDeleteAll() {
	confirmingAction.value = 'delete-all'
	restoreResult.value = null
	actionError.value = null
}

function cancelConfirm() {
	confirmingId.value = null
	confirmingAction.value = null
}

async function confirmAction() {
	const action = confirmingAction.value
	const backupId = confirmingId.value
	if (!action || (action !== 'delete-all' && !backupId)) return
	confirmingId.value = null
	confirmingAction.value = null
	busy.value = true
	actionError.value = null
	restoreResult.value = null
	try {
		if (action === 'restore') {
			const count = await api.restoreBackup(backupId as string)
			restoreResult.value = { backupId: backupId as string, count }
		} else if (action === 'delete') {
			await api.deleteBackup(backupId as string)
			backups.value = backups.value.filter((backup) => backup.id !== backupId)
		} else {
			await api.deleteAllBackups()
			backups.value = []
		}
	} catch (e: unknown) {
		const fallback =
			action === 'restore'
				? t('settings.backups.errors.restoreFailed')
				: action === 'delete'
					? t('settings.backups.errors.deleteFailed')
					: t('settings.backups.errors.deleteAllFailed')
		actionError.value = (e as { message?: string })?.message ?? fallback
	} finally {
		busy.value = false
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader :title="t('settings.backups.title')" />
		<div class="flex flex-col gap-2">
			<OptionButton
				:icon="Trash2"
				:label="t('settings.backups.deleteAll.label')"
				:description="t('settings.backups.deleteAll.description')"
				:button-label="t('settings.backups.deleteAll.buttonLabel')"
				:disabled="busy || backups.length === 0"
				@click="startDeleteAll"
			/>

			<div class="overflow-hidden rounded-lg border border-border bg-surface-3">
				<div v-if="backupsLoading" class="flex items-center gap-2 px-4 py-3 text-sm text-secondary">
					<Loader2 :size="15" class="animate-spin" />
					{{ t('settings.backups.loading') }}
				</div>

				<p v-else-if="backups.length === 0" class="px-4 py-3 text-sm text-secondary">
					{{ t('settings.backups.noBackups') }}
				</p>

				<div v-else class="max-h-112 divide-y divide-border/50 overflow-y-auto">
					<div
						v-for="backup in backups"
						:key="backup.id"
						class="flex items-center gap-3 px-4 py-2.5"
					>
						<div class="min-w-0 flex-1">
							<p class="text-sm font-medium">{{ formatBackupDate(backup.createdAt) }}</p>
							<p class="mt-0.5 text-xs text-secondary">
								{{ t('settings.backups.fileCount', { count: backup.fileCount }, backup.fileCount) }}
								·
								{{ formatSize(backup.sizeBytes) }}
							</p>
						</div>
						<UiButton variant="ghost" :disabled="busy" @click="startRestore(backup.id)">
							<RotateCcw :size="14" />
							{{ t('settings.backups.restore') }}
						</UiButton>
						<UiButton
							size="icon"
							variant="ghost"
							:title="t('settings.backups.deleteBackupTitle')"
							:disabled="busy"
							@click="startDelete(backup.id)"
						>
							<Trash2 :size="14" />
						</UiButton>
					</div>
				</div>

				<div v-if="restoreResult || actionError" class="border-t border-border px-4 py-3">
					<div v-if="restoreResult" class="flex items-center gap-2 text-sm">
						<CheckCircle2 :size="15" class="shrink-0 text-accent" />
						{{
							t(
								'settings.backups.restoredMessage',
								{ count: restoreResult.count },
								restoreResult.count,
							)
						}}
					</div>
					<div v-if="actionError" class="flex items-center gap-2 text-sm text-danger">
						<AlertCircle :size="15" class="shrink-0" />
						{{ actionError }}
					</div>
				</div>
			</div>
		</div>
	</section>

	<Modal
		:model-value="confirmingId !== null || confirmingAction === 'delete-all'"
		@update:model-value="cancelConfirm"
	>
		<div class="mb-5 flex items-start gap-3">
			<AlertTriangle :size="20" class="mt-0.5 shrink-0 text-warning" />
			<div>
				<h2 class="mb-1.5 text-sm font-semibold">
					{{
						confirmingAction === 'delete-all'
							? t('settings.backups.confirm.deleteAllTitle')
							: confirmingAction === 'delete'
								? t('settings.backups.confirm.deleteTitle')
								: t('settings.backups.confirm.restoreTitle')
					}}
				</h2>
				<p class="text-xs text-secondary">
					<template v-if="confirmingAction === 'delete-all'">
						{{
							t('settings.backups.confirm.deleteAllBody', { count: backups.length }, backups.length)
						}}
					</template>
					<template v-else-if="confirmingAction === 'delete'">
						{{
							t('settings.backups.confirm.deleteBody', {
								date: confirmingBackup ? formatBackupDate(confirmingBackup.createdAt) : '',
							})
						}}
					</template>
					<template v-else>
						{{
							t('settings.backups.confirm.restoreBody', {
								date: confirmingBackup ? formatBackupDate(confirmingBackup.createdAt) : '',
							})
						}}
					</template>
				</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton variant="ghost" @click="cancelConfirm">{{ t('common.cancel') }}</UiButton>
			<UiButton variant="danger" @click="confirmAction">
				{{
					confirmingAction === 'delete-all'
						? t('settings.backups.confirm.confirmDeleteAll')
						: confirmingAction === 'delete'
							? t('settings.backups.confirm.confirmDelete')
							: t('settings.backups.confirm.confirmRestore')
				}}
			</UiButton>
		</div>
	</Modal>
</template>
