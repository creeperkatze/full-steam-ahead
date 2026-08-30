<script setup lang="ts">
import { Check, Loader2, RotateCw, Search } from '@lucide/vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import SourceIcon from '../../../components/SourceIcon.vue'
import UiButton from '../../../components/ui/Button.vue'
import Dropdown from '../../../components/ui/Dropdown.vue'
import ItemRow from '../../../components/ui/ItemRow.vue'
import UserAvatar from '../../../components/UserAvatar.vue'
import { useAppState } from '../../../composables/useAppState'
import { useScanSources } from '../../../composables/useScanSources'
import { useTaskStatus } from '../../../composables/useTaskStatus'
import { api } from '../../../helpers/api'

const state = useAppState()
const task = useTaskStatus()
const { sourceStates } = useScanSources()
const { t } = useI18n()

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
const scanProgressPct = computed(() =>
	sourceStates.value.length > 0 ? (doneCount.value / sourceStates.value.length) * 100 : 0,
)

onMounted(async () => {
	if (!state.install.value) {
		await refreshSteam()
	}
})

function steamUserName(user: { accountName?: string | null }) {
	return user.accountName?.trim() || t('startView.unnamedUser')
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
		<section
			v-if="state.scanPhase.value !== 'scanning'"
			class="flex flex-1 flex-col items-center justify-center gap-6 rounded-lg border border-accent/30 bg-accent-bg px-8 py-8 text-center"
		>
			<div class="grid size-16 place-items-center rounded-full bg-accent text-accent-icon">
				<Search :size="28" />
			</div>

			<div>
				<h1 class="text-2xl font-bold">{{ t('startView.title') }}</h1>
				<p class="mt-1 text-secondary">{{ t('startView.subtitle') }}</p>
			</div>

			<!-- Loading Steam -->
			<div v-if="task.loading.value" class="flex items-center gap-2 text-sm text-secondary">
				<Loader2 :size="14" class="animate-spin" />
				{{ t('startView.detectingSteam') }}
			</div>

			<!-- Steam not found -->
			<div v-else-if="!state.install.value" class="flex flex-col items-center gap-2">
				<p class="text-sm text-danger">
					{{ t('startView.steamNotFound') }}
				</p>
				<UiButton size="sm" variant="ghost" @click="refreshSteam">
					<RotateCw :size="14" />
					{{ t('startView.tryAgain') }}
				</UiButton>
			</div>

			<!-- No users -->
			<p v-else-if="steamUsers.length === 0" class="text-sm text-danger">
				{{ t('startView.noUsersFound') }}
			</p>

			<!-- Ready: user selector -->
			<template v-else>
				<div
					class="flex items-center gap-3 rounded-lg border border-border bg-surface-3 px-4 py-2.5"
				>
					<span class="shrink-0 text-sm text-secondary">{{ t('startView.steamUser') }}</span>
					<Dropdown v-model="state.selectedUserId.value" :options="userOptions">
						<template #leading="{ option }">
							<UserAvatar :src="option.avatarSrc" :size="18" />
						</template>
					</Dropdown>
				</div>
			</template>
		</section>
		<section v-else class="overflow-hidden rounded-lg border border-border">
			<div
				class="flex items-center justify-between border-b border-border bg-surface-4 px-3 py-2.5"
			>
				<div>
					<h1 class="text-base font-bold">{{ t('startView.scanningTitle') }}</h1>
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
							{{
								s.found > 0
									? t('startView.sourceFound', { count: s.found })
									: t('startView.sourcePending')
							}}
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
						{{ t('startView.sourcesScanned', { done: doneCount, total: sourceStates.length }) }}
					</p>
				</div>
			</div>
		</section>
	</div>
</template>
