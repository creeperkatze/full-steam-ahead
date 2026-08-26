<script setup lang="ts">
import {
	AlertCircle,
	AlertTriangle,
	Archive,
	CheckCircle2,
	Gamepad2,
	HardDrive,
	Layers,
	Loader2,
	Power,
	RotateCcw,
	RotateCw,
	Trash2,
} from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, ref } from 'vue'

import GitHubIcon from '../../../assets/icons/github.svg?component'
import KofiIcon from '../../../assets/icons/kofi.svg?component'
import Card from '../../../components/Card.vue'
import Modal from '../../../components/Modal.vue'
import OptionPath from '../../../components/options/OptionPath.vue'
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
const confirmingAction = ref<'restore' | 'delete' | null>(null)
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

async function pickSteamLocation() {
	const picked = await open({ directory: true, multiple: false })
	if (typeof picked === 'string') {
		state.steamLocation.value = picked
	}
}

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

function cancelConfirm() {
	confirmingId.value = null
	confirmingAction.value = null
}

async function confirmAction() {
	if (!confirmingId.value || !confirmingAction.value) return
	const backupId = confirmingId.value
	const action = confirmingAction.value
	confirmingId.value = null
	confirmingAction.value = null
	busy.value = true
	actionError.value = null
	restoreResult.value = null
	try {
		if (action === 'restore') {
			const count = await api.restoreBackup(backupId)
			restoreResult.value = { backupId, count }
		} else {
			await api.deleteBackup(backupId)
			backups.value = backups.value.filter((backup) => backup.id !== backupId)
		}
	} catch (e: unknown) {
		const fallback = action === 'restore' ? 'Restore failed.' : 'Delete failed.'
		actionError.value = (e as { message?: string })?.message ?? fallback
	} finally {
		busy.value = false
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
					:icon="GitHubIcon"
					color="#9ca3af"
					title="View on GitHub"
					description="Leave a star"
				/>
			</div>
		</nav>

		<div class="min-w-0 flex-1 border-l border-border pl-4">
			<!-- Steam -->
			<section v-if="activeSection === 'steam'" class="max-w-2xl">
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
					<OptionToggle
						v-model="state.options.value.createCollections"
						:icon="Layers"
						label="Create per-platform collections"
						description="Groups imported games into a collection for each launcher"
					/>
					<OptionPath
						v-model="state.steamLocation.value"
						:icon="HardDrive"
						label="Steam installation location"
						description="Override auto-detection if Steam isn't found automatically"
						placeholder="Auto-detected"
						@browse="pickSteamLocation"
					/>
				</div>
			</section>

			<!-- Backups -->
			<section v-else-if="activeSection === 'backups'" class="max-w-2xl">
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
						<div
							v-for="backup in backups"
							:key="backup.id"
							class="flex items-center gap-3 px-4 py-2.5"
						>
							<div class="min-w-0 flex-1">
								<p class="font-mono text-sm">{{ formatBackupDate(backup.createdAt) }}</p>
								<p class="text-xs text-secondary">
									{{ backup.fileCount }} {{ backup.fileCount === 1 ? 'file' : 'files' }} ·
									{{ formatSize(backup.sizeBytes) }}
								</p>
							</div>
							<UiButton size="sm" variant="ghost" :disabled="busy" @click="startRestore(backup.id)">
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
			</section>
		</div>
	</div>

	<Modal :model-value="confirmingId !== null" @update:model-value="cancelConfirm">
		<div class="mb-5 flex items-start gap-3">
			<AlertTriangle :size="20" class="mt-0.5 shrink-0 text-warning" />
			<div>
				<h2 class="mb-1.5 text-sm font-semibold">
					{{ confirmingAction === 'delete' ? 'Delete backup?' : 'Restore backup?' }}
				</h2>
				<p class="text-xs text-secondary">
					<template v-if="confirmingAction === 'delete'">
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
				{{ confirmingAction === 'delete' ? 'Yes, delete' : 'Yes, restore' }}
			</UiButton>
		</div>
	</Modal>
</template>
