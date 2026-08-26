<script setup lang="ts">
import {
	AlertCircle,
	Archive,
	CheckCircle2,
	Gamepad2,
	Loader2,
	Power,
	RotateCcw,
	RotateCw,
	Star,
} from '@lucide/vue'
import { onMounted, ref } from 'vue'

import KofiIcon from '../../../assets/icons/kofi.svg?component'
import Card from '../../../components/Card.vue'
import OptionToggle from '../../../components/options/OptionToggle.vue'
import SectionHeader from '../../../components/options/SectionHeader.vue'
import SidebarTab from '../../../components/options/SidebarTab.vue'
import UiButton from '../../../components/ui/Button.vue'
import { useAppState } from '../../../composables/useAppState'
import { api } from '../../../helpers/api'
import type { BackupInfo } from '../../../types'

const state = useAppState()

const sections = [
	{ id: 'steam', label: 'Steam', icon: Gamepad2 },
	{ id: 'backups', label: 'Backups', icon: Archive },
] as const
const activeSection = ref<(typeof sections)[number]['id']>('steam')

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
	<div class="flex min-h-0 flex-1">
		<nav class="flex w-48 shrink-0 flex-col gap-1 pr-4">
			<SidebarTab
				v-for="section in sections"
				:key="section.id"
				:icon="section.icon"
				:label="section.label"
				:active="activeSection === section.id"
				@click="activeSection = section.id"
			/>

			<div class="mt-auto flex flex-col gap-1.5 pt-4">
				<Card
					href="https://ko-fi.com/creeperkatze"
					:icon="KofiIcon"
					color="#FF5E5B"
					title="Donate on Ko-fi"
					description="Buy me a coffee"
				/>
				<Card
					href="https://github.com/creeperkatze/full-steam-ahead"
					:icon="Star"
					color="#9ca3af"
					title="View on GitHub"
					description="Leave a star"
				/>
			</div>
		</nav>

		<div class="min-w-0 flex-1 border-l border-border pl-4">
			<!-- Steam -->
			<section v-if="activeSection === 'steam'" class="max-w-sm">
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
				</div>
			</section>

			<!-- Backups -->
			<section v-else-if="activeSection === 'backups'" class="max-w-sm">
				<SectionHeader title="Backups" />
				<div class="overflow-hidden rounded-lg border border-border bg-surface-3">
					<div
						v-if="backupsLoading"
						class="flex items-center gap-2 px-4 py-3 text-sm text-secondary"
					>
						<Loader2 :size="15" class="animate-spin" />
						Loading backups…
					</div>

					<p v-else-if="backups.length === 0" class="px-4 py-3 text-sm text-secondary">
						No backups found.
					</p>

					<div v-else class="max-h-72 divide-y divide-border/50 overflow-y-auto">
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
									<UiButton size="sm" variant="danger" @click="confirmRestore">
										Yes, restore
									</UiButton>
									<UiButton size="sm" variant="ghost" @click="cancelRestore">Cancel</UiButton>
								</div>
							</div>
						</div>
					</div>

					<div v-if="restoreResult || restoreError" class="border-t border-border px-4 py-3">
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
				</div>
			</section>
		</div>
	</div>
</template>
