<script setup lang="ts">
import { Check, Loader2, Search } from '@lucide/vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onMounted } from 'vue'

import SourceIcon from '../../../components/SourceIcon.vue'
import UserAvatar from '../../../components/UserAvatar.vue'
import Dropdown from '../../../components/ui/Dropdown.vue'
import ItemRow from '../../../components/ui/ItemRow.vue'
import { useAppState } from '../../../composables/useAppState'
import { SCANNABLE_SOURCES, useScanSources } from '../../../composables/useScanSources'
import { useTaskStatus } from '../../../composables/useTaskStatus'
import { api } from '../../../helpers/api'

const state = useAppState()
const task = useTaskStatus()
const { sourceStates } = useScanSources()

const steamUsers = computed(() =>
	[...(state.install.value?.users ?? [])].sort((a, b) =>
		steamUserName(a).localeCompare(steamUserName(b)),
	),
)

const userOptions = computed(() =>
	steamUsers.value.map((user) => ({
		value: user.steamId,
		label: steamUserName(user),
		avatarSrc: user.avatarPath ? convertFileSrc(user.avatarPath) : null,
	})),
)

const doneCount = computed(() => sourceStates.value.filter((s) => s.status === 'done').length)
const foundTotal = computed(() =>
	sourceStates.value.reduce((sum, s) => sum + (s.status === 'done' ? s.found : 0), 0),
)
const scanProgressPct = computed(() =>
	SCANNABLE_SOURCES.length > 0 ? (doneCount.value / SCANNABLE_SOURCES.length) * 100 : 0,
)

onMounted(async () => {
	if (!state.install.value) {
		await refreshSteam()
	}
})

function steamUserName(user: { accountName?: string | null }) {
	return user.accountName?.trim() || 'Unnamed Steam User'
}

async function refreshSteam() {
	const detected = await task.runTask('Detecting Steam', () => api.detectSteam())
	if (!detected) return

	state.install.value = detected
	state.selectedUserId.value = detected.users[0]?.steamId ?? ''
	state.invalidatePreview()
}
</script>

<template>
	<div class="flex flex-1 flex-col gap-4">
		<!-- ── Welcome (idle / done) ──────────────────────────────────── -->
		<section
			v-if="state.scanPhase.value !== 'scanning'"
			class="flex flex-1 flex-col items-center justify-center gap-6 rounded-xl border border-accent/30 bg-accent-bg px-8 py-8 text-center"
		>
			<div class="grid size-16 place-items-center rounded-full bg-accent text-accent-contrast">
				<Search :size="28" />
			</div>

			<div>
				<h1 class="text-2xl font-bold">Find your games</h1>
				<p class="mt-1 text-secondary">Scan your installed launchers to import them into Steam.</p>
			</div>

			<!-- Loading Steam -->
			<div v-if="task.loading.value" class="flex items-center gap-2 text-sm text-secondary">
				<Loader2 :size="14" class="animate-spin" />
				Detecting Steam installation…
			</div>

			<!-- Steam not found -->
			<p v-else-if="!state.install.value" class="text-sm text-danger">
				Steam installation not found.
				<button class="ml-1 underline hover:no-underline" @click="refreshSteam">Try again</button>
			</p>

			<!-- No users -->
			<p v-else-if="steamUsers.length === 0" class="text-sm text-danger">No Steam users found.</p>

			<!-- Ready: user selector -->
			<template v-else>
				<div
					class="flex items-center gap-3 rounded-lg border border-border bg-surface-3 px-4 py-2.5"
				>
					<span class="shrink-0 text-sm text-secondary">Steam User</span>
					<Dropdown v-model="state.selectedUserId.value" :options="userOptions">
						<template #leading="{ option }">
							<UserAvatar :src="option.avatarSrc" :size="18" />
						</template>
					</Dropdown>
				</div>
			</template>
		</section>

		<!-- ── Scanning (progress) ─────────────────────────────────────── -->
		<section v-else class="overflow-hidden rounded-xl border border-border">
			<div
				class="flex items-center justify-between border-b border-border bg-surface-4 px-3 py-2.5"
			>
				<div>
					<h1 class="text-base font-bold">Scanning for games…</h1>
					<p class="text-sm text-secondary">
						{{ foundTotal }} game{{ foundTotal !== 1 ? 's' : '' }} found so far
					</p>
				</div>
				<Loader2 :size="20" class="animate-spin text-accent" />
			</div>

			<div class="grid gap-1.5 bg-surface-3 p-2">
				<ItemRow v-for="s in sourceStates" :key="s.key" :active="s.status === 'scanning'">
					<template #leading>
						<Check v-if="s.status === 'done'" :size="14" class="shrink-0 text-accent" />
						<Loader2
							v-else-if="s.status === 'scanning'"
							:size="14"
							class="shrink-0 animate-spin text-accent"
						/>
						<div v-else class="size-3.5 shrink-0 rounded-full border border-border-muted" />
						<SourceIcon :source="s.key" class="size-4 shrink-0" />
					</template>

					<span :class="s.status === 'pending' ? 'text-secondary' : 'font-medium'">{{
						s.name
					}}</span>

					<template #trailing>
						<span v-if="s.status === 'done'" class="shrink-0 text-xs text-secondary">
							{{ s.found > 0 ? `${s.found} found` : 'none' }}
						</span>
					</template>
				</ItemRow>

				<div class="space-y-1.5 px-1 pb-1 pt-0.5">
					<div class="h-1.5 overflow-hidden rounded-full bg-surface-5">
						<div
							class="h-full rounded-full bg-accent transition-all duration-500"
							:style="{ width: `${scanProgressPct}%` }"
						/>
					</div>
					<p class="text-xs text-secondary">
						{{ doneCount }} of {{ SCANNABLE_SOURCES.length }} sources scanned
					</p>
				</div>
			</div>
		</section>
	</div>
</template>
