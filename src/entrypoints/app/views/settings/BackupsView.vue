<script setup lang="ts">
import { AlertCircle, AlertTriangle, CheckCircle2, Loader2, RotateCcw, Trash2 } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'

import Modal from '../../../../components/Modal.vue'
import OptionButton from '../../../../components/options/OptionButton.vue'
import SectionHeader from '../../../../components/options/SectionHeader.vue'
import UiButton from '../../../../components/ui/Button.vue'
import { api } from '../../../../helpers/api'
import type { BackupInfo } from '../../../../types'

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
				? 'Restore failed.'
				: action === 'delete'
					? 'Delete failed.'
					: 'Delete all failed.'
		actionError.value = (e as { message?: string })?.message ?? fallback
	} finally {
		busy.value = false
	}
}
</script>

<template>
	<section class="max-w-2xl">
		<SectionHeader title="Backups" />
		<div class="flex flex-col gap-2">
			<OptionButton
				:icon="Trash2"
				label="Delete all backups"
				description="Permanently remove every stored backup."
				button-label="Delete all"
				:disabled="busy || backups.length === 0"
				@click="startDeleteAll"
			/>

			<div class="overflow-hidden rounded-lg border border-border bg-surface-3">
				<div v-if="backupsLoading" class="flex items-center gap-2 px-4 py-3 text-sm text-secondary">
					<Loader2 :size="15" class="animate-spin" />
					Loading backups…
				</div>

				<p v-else-if="backups.length === 0" class="px-4 py-3 text-sm text-secondary">
					No backups found.
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
								{{ backup.fileCount }} {{ backup.fileCount === 1 ? 'file' : 'files' }} ·
								{{ formatSize(backup.sizeBytes) }}
							</p>
						</div>
						<UiButton variant="ghost" :disabled="busy" @click="startRestore(backup.id)">
							<RotateCcw :size="14" />
							Restore
						</UiButton>
						<UiButton
							size="icon"
							variant="ghost"
							title="Delete backup"
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
						Restored {{ restoreResult.count }}
						{{ restoreResult.count === 1 ? 'file' : 'files' }} successfully.
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
							? 'Delete all backups?'
							: confirmingAction === 'delete'
								? 'Delete backup?'
								: 'Restore backup?'
					}}
				</h2>
				<p class="text-xs text-secondary">
					<template v-if="confirmingAction === 'delete-all'">
						This will permanently delete all {{ backups.length }}
						{{ backups.length === 1 ? 'backup' : 'backups' }}. This cannot be undone.
					</template>
					<template v-else-if="confirmingAction === 'delete'">
						This will permanently delete the backup from
						{{ confirmingBackup ? formatBackupDate(confirmingBackup.createdAt) : '' }}. This cannot
						be undone.
					</template>
					<template v-else>
						This will overwrite the current Steam files for this account with the backup from
						{{ confirmingBackup ? formatBackupDate(confirmingBackup.createdAt) : '' }}. This cannot
						be undone.
					</template>
				</p>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton variant="ghost" @click="cancelConfirm">Cancel</UiButton>
			<UiButton variant="danger" @click="confirmAction">
				{{
					confirmingAction === 'delete-all'
						? 'Yes, delete all'
						: confirmingAction === 'delete'
							? 'Yes, delete'
							: 'Yes, restore'
				}}
			</UiButton>
		</div>
	</Modal>
</template>
