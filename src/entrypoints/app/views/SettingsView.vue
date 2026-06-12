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
	return iso.replace('T', ' ').replace('Z', '')
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
	<section class="grid w-full grid-cols-2 items-start gap-3">
		<section class="rounded-lg border border-border bg-surface-3 p-4">
			<h2 class="mb-3 text-base font-semibold">Apply Options</h2>
			<div class="grid gap-2">
				<label
					class="flex min-h-10 cursor-pointer items-center gap-2 rounded-md border border-danger-border bg-surface-5 px-3 text-danger"
				>
					<UiCheckbox v-model="state.options.value.stopSteam" />
					Stop Steam before applying
				</label>
				<label
					class="flex min-h-10 cursor-pointer items-center gap-2 rounded-md border border-danger-border bg-surface-5 px-3 text-danger"
				>
					<UiCheckbox v-model="state.options.value.restartSteam" />
					Restart Steam after applying
				</label>
			</div>
		</section>

		<section class="rounded-lg border border-border bg-surface-3 p-4">
			<h2 class="mb-1 text-base font-semibold">Restore Backup</h2>
			<p class="mb-3 text-sm text-secondary">
				Restore Steam shortcuts, collections, and artwork to the state of a previous backup.
			</p>

			<!-- Loading -->
			<div v-if="backupsLoading" class="flex items-center gap-2 text-sm text-secondary">
				<Loader2 :size="15" class="animate-spin" />
				Loading backups…
			</div>

			<!-- No backups -->
			<p v-else-if="backups.length === 0" class="text-sm text-secondary">No backups found.</p>

			<!-- Backup list -->
			<div v-else class="grid gap-2">
				<div
					v-for="backup in backups"
					:key="backup.id"
					class="rounded-md border border-border bg-surface-5"
				>
					<div class="flex items-center gap-3 px-3 py-2">
						<div class="min-w-0 flex-1">
							<span class="font-mono text-sm">{{ formatBackupDate(backup.createdAt) }}</span>
							<span class="ml-2 text-xs text-secondary">
								{{ backup.fileCount }}
								{{ backup.fileCount === 1 ? 'file' : 'files' }} ·
								{{ formatSize(backup.sizeBytes) }}
							</span>
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

					<!-- Inline confirmation -->
					<div v-if="confirmingId === backup.id" class="border-t border-border px-3 py-2">
						<p class="mb-2 text-sm text-secondary">
							This will overwrite the current Steam files for this account. Continue?
						</p>
						<div class="flex gap-2">
							<UiButton size="sm" variant="danger" @click="confirmRestore"> Yes, restore </UiButton>
							<UiButton size="sm" variant="ghost" @click="cancelRestore">Cancel</UiButton>
						</div>
					</div>
				</div>
			</div>

			<!-- Restore feedback -->
			<div
				v-if="restoreResult"
				class="mt-3 flex items-center gap-2 rounded-md border border-border bg-surface-5 px-3 py-2 text-sm"
			>
				<CheckCircle2 :size="15" class="shrink-0 text-accent" />
				Restored {{ restoreResult.count }}
				{{ restoreResult.count === 1 ? 'file' : 'files' }} successfully.
			</div>
			<div
				v-if="restoreError"
				class="mt-3 flex items-center gap-2 rounded-md border border-danger-border bg-surface-5 px-3 py-2 text-sm text-danger"
			>
				<AlertCircle :size="15" class="shrink-0" />
				{{ restoreError }}
			</div>
		</section>
	</section>
</template>
