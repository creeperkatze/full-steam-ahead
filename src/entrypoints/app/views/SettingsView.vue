<script setup lang="ts">
import { AlertCircle, CheckCircle2, Loader2, RotateCcw } from '@lucide/vue'
import { onMounted, ref } from 'vue'

import UiButton from '../../../components/ui/Button.vue'
import UiCheckbox from '../../../components/ui/Checkbox.vue'
import { useAppState } from '../../../composables/useAppState'
import { api } from '../../../helpers/api'
import type { BackupInfo } from '../../../types'

const state = useAppState()

const backups = ref<BackupInfo[]>([])
const backupsLoading = ref(true)
const confirmingId = ref<string | null>(null)
const restoring = ref(false)
const restoreResult = ref<{ backupId: string; count: number } | null>(null)
const restoreError = ref<string | null>(null)

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
	return new Date(iso).toLocaleString()
}

function formatSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`
	if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`
	return `${(bytes / 1_048_576).toFixed(1)} MB`
}

function startRestore(backupId: string) {
	confirmingId.value = backupId
	restoreResult.value = null
	restoreError.value = null
}

function cancelRestore() {
	confirmingId.value = null
}

async function confirmRestore() {
	if (!confirmingId.value) return
	const backupId = confirmingId.value
	confirmingId.value = null
	restoring.value = true
	restoreError.value = null
	restoreResult.value = null
	try {
		const count = await api.restoreBackup(backupId)
		restoreResult.value = { backupId, count }
	} catch (e: unknown) {
		restoreError.value = (e as { message?: string })?.message ?? 'Restore failed.'
	} finally {
		restoring.value = false
	}
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<!-- Steam -->
		<section class="overflow-hidden rounded-xl border border-border">
			<div class="border-b border-border bg-surface-4 px-4 py-3">
				<h2 class="font-semibold">Steam</h2>
			</div>
			<div class="divide-y divide-border/50 bg-surface-3">
				<label class="flex cursor-pointer items-center gap-3 px-4 py-3 hover:bg-surface-4">
					<UiCheckbox v-model="state.options.value.stopSteam" />
					<div>
						<p class="text-sm font-medium">Stop Steam before applying</p>
						<p class="text-xs text-secondary">Steam must be closed to modify shortcut files</p>
					</div>
				</label>
				<label class="flex cursor-pointer items-center gap-3 px-4 py-3 hover:bg-surface-4">
					<UiCheckbox v-model="state.options.value.restartSteam" />
					<div>
						<p class="text-sm font-medium">Restart Steam after applying</p>
						<p class="text-xs text-secondary">
							Relaunches Steam so imported games appear immediately
						</p>
					</div>
				</label>
			</div>
		</section>

		<!-- Backups -->
		<section class="overflow-hidden rounded-xl border border-border">
			<div class="flex items-center justify-between border-b border-border bg-surface-4 px-4 py-3">
				<h2 class="font-semibold">Backups</h2>
				<span v-if="!backupsLoading" class="text-xs text-secondary">
					{{ backups.length }} {{ backups.length === 1 ? 'backup' : 'backups' }}
				</span>
			</div>

			<div
				v-if="backupsLoading"
				class="flex items-center gap-2 bg-surface-3 px-4 py-3 text-sm text-secondary"
			>
				<Loader2 :size="15" class="animate-spin" />
				Loading backups…
			</div>

			<p v-else-if="backups.length === 0" class="bg-surface-3 px-4 py-3 text-sm text-secondary">
				No backups found.
			</p>

			<div v-else class="max-h-72 divide-y divide-border/50 overflow-y-auto bg-surface-3">
				<div v-for="backup in backups" :key="backup.id">
					<div class="flex items-center gap-3 px-4 py-2.5">
						<div class="min-w-0 flex-1">
							<p class="font-mono text-sm">{{ formatBackupDate(backup.createdAt) }}</p>
							<p class="text-xs text-secondary">
								{{ backup.fileCount }} {{ backup.fileCount === 1 ? 'file' : 'files' }} ·
								{{ formatSize(backup.sizeBytes) }}
							</p>
						</div>
						<UiButton
							v-if="confirmingId !== backup.id"
							size="sm"
							variant="ghost"
							:disabled="restoring"
							@click="startRestore(backup.id)"
						>
							<RotateCcw :size="14" />
							Restore
						</UiButton>
					</div>

					<div
						v-if="confirmingId === backup.id"
						class="border-t border-border/50 bg-surface-5 px-4 py-2.5"
					>
						<p class="mb-2 text-sm text-secondary">
							This will overwrite the current Steam files for this account. Continue?
						</p>
						<div class="flex gap-2">
							<UiButton size="sm" variant="danger" @click="confirmRestore">Yes, restore</UiButton>
							<UiButton size="sm" variant="ghost" @click="cancelRestore">Cancel</UiButton>
						</div>
					</div>
				</div>
			</div>

			<div
				v-if="restoreResult || restoreError"
				class="border-t border-border bg-surface-3 px-4 py-3"
			>
				<div v-if="restoreResult" class="flex items-center gap-2 text-sm">
					<CheckCircle2 :size="15" class="shrink-0 text-accent" />
					Restored {{ restoreResult.count }}
					{{ restoreResult.count === 1 ? 'file' : 'files' }} successfully.
				</div>
				<div v-if="restoreError" class="flex items-center gap-2 text-sm text-danger">
					<AlertCircle :size="15" class="shrink-0" />
					{{ restoreError }}
				</div>
			</div>
		</section>
	</div>
</template>
