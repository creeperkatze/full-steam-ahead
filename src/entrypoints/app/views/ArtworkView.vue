<script setup lang="ts">
import { FolderPlus, RotateCcw, Trash2 } from '@lucide/vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import SteamGridDbIcon from '../../../assets/icons/steamgriddb.svg'
import GameIcon from '../../../components/GameIcon.vue'
import SteamGridDbBrowser from '../../../components/SteamGridDbBrowser.vue'
import UiButton from '../../../components/ui/Button.vue'
import { useAppState } from '../../../composables/useAppState'
import type { ArtworkAsset, ArtworkKind, ImportCandidate, SteamGridDbImage } from '../../../types'

const state = useAppState()
const { t } = useI18n()

const slots = computed<Array<{ kind: ArtworkKind; label: string }>>(() => [
	{ kind: 'header', label: t('artworkView.slots.header') },
	{ kind: 'capsule', label: t('artworkView.slots.capsule') },
	{ kind: 'hero', label: t('artworkView.slots.hero') },
	{ kind: 'logo', label: t('artworkView.slots.logo') },
	{ kind: 'icon', label: t('artworkView.slots.icon') },
])

const brokenPreviewUrls = ref<Record<string, true>>({})

const steamGridDbAvailable = computed(
	() => state.settings.steamGridDb.enabled && !!state.settings.steamGridDb.apiKey,
)

const browsingSlot = ref<{ candidateId: string; kind: ArtworkKind; name: string } | null>(null)

function artworkKey(candidateId: string, kind: ArtworkKind) {
	return `${candidateId}:${kind}`
}

function selectedAsset(candidate: ImportCandidate, kind: ArtworkKind): ArtworkAsset | undefined {
	const localPath = state.customArtwork.value[artworkKey(candidate.id, kind)]
	if (localPath) {
		return {
			kind,
			pathOrUrl: localPath,
			source: 'localFile',
			willReplaceExisting: true,
		}
	}
	const matches = candidate.artwork.proposed.filter((asset) => asset.kind === kind)
	return (
		matches.find((asset) => asset.source === 'missing') ??
		matches.find((asset) => asset.source === 'steamGridDb') ??
		matches.find((asset) => asset.source === 'officialSteam') ??
		matches[0]
	)
}

function existingAsset(candidate: ImportCandidate, kind: ArtworkKind): ArtworkAsset | undefined {
	return candidate.artwork.existing.find((asset) => asset.kind === kind)
}

function sourceLabel(asset?: ArtworkAsset) {
	if (!asset) return t('artworkSource.missing')
	if (asset.source === 'officialSteam') return t('artworkSource.officialSteam')
	if (asset.source === 'localFile') return t('artworkSource.localFile')
	if (asset.source === 'existingCustom') return t('artworkSource.existingCustom')
	if (asset.source === 'missing') return t('artworkSource.missing')
	return t('artworkSource.steamGridDb')
}

function previewSrc(asset?: ArtworkAsset) {
	if (!asset) return ''
	return asset.source === 'localFile' || asset.source === 'existingCustom'
		? convertFileSrc(asset.pathOrUrl)
		: asset.pathOrUrl
}

function previewErrored(asset?: ArtworkAsset) {
	const src = previewSrc(asset)
	return src ? brokenPreviewUrls.value[src] : false
}

function markPreviewErrored(asset?: ArtworkAsset) {
	const src = previewSrc(asset)
	if (src) brokenPreviewUrls.value[src] = true
}

function displayAsset(candidate: ImportCandidate, kind: ArtworkKind) {
	return selectedAsset(candidate, kind) || existingAsset(candidate, kind)
}

async function pickArtwork(candidateId: string, kind: ArtworkKind) {
	const picked = await open({
		multiple: false,
		filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
	})
	if (typeof picked !== 'string') return

	removeArtworkOverride(candidateId, kind)
	state.customArtwork.value = {
		...state.customArtwork.value,
		[artworkKey(candidateId, kind)]: picked,
	}
	upsertArtworkAsset(candidateId, {
		kind,
		pathOrUrl: picked,
		source: 'localFile',
		willReplaceExisting: true,
	})
}

function openSteamGridDbBrowser(candidate: ImportCandidate, kind: ArtworkKind) {
	browsingSlot.value = { candidateId: candidate.id, kind, name: candidate.name }
}

function onSteamGridDbSelect(image: SteamGridDbImage) {
	if (!browsingSlot.value) return
	const { candidateId, kind } = browsingSlot.value
	removeArtworkOverride(candidateId, kind)
	const updatedCustomArtwork = { ...state.customArtwork.value }
	delete updatedCustomArtwork[artworkKey(candidateId, kind)]
	state.customArtwork.value = updatedCustomArtwork
	upsertArtworkAsset(candidateId, {
		kind,
		pathOrUrl: image.url,
		source: 'steamGridDb',
		willReplaceExisting: true,
	})
	browsingSlot.value = null
}

function deleteArtwork(candidateId: string, kind: ArtworkKind) {
	removeArtworkOverride(candidateId, kind)
	const updated = { ...state.customArtwork.value }
	delete updated[artworkKey(candidateId, kind)]
	state.customArtwork.value = updated
	upsertArtworkAsset(candidateId, {
		kind,
		pathOrUrl: '',
		source: 'missing',
		willReplaceExisting: true,
	})
}

function useOfficialArtwork(candidateId: string, kind: ArtworkKind) {
	const candidate = state.candidates.value.find((candidate) => candidate.id === candidateId)
	const official = candidate?.artwork.proposed.find(
		(asset) => asset.kind === kind && asset.source === 'officialSteam',
	)
	if (!official) return

	removeArtworkOverride(candidateId, kind)
	const updated = { ...state.customArtwork.value }
	delete updated[artworkKey(candidateId, kind)]
	state.customArtwork.value = updated
}

function upsertArtworkAsset(candidateId: string, asset: ArtworkAsset) {
	state.candidates.value = state.candidates.value.map((candidate) => {
		if (candidate.id !== candidateId) return candidate
		const proposed = candidate.artwork.proposed.filter(
			(item) => !(item.kind === asset.kind && item.source === asset.source),
		)
		return {
			...candidate,
			artwork: {
				...candidate.artwork,
				mode:
					asset.source === 'localFile' ||
					asset.source === 'steamGridDb' ||
					asset.source === 'missing'
						? 'localOverride'
						: candidate.artwork.mode,
				proposed: [...proposed, asset],
			},
		}
	})
	state.invalidatePreview()
}

function removeArtworkOverride(candidateId: string, kind: ArtworkKind) {
	state.candidates.value = state.candidates.value.map((candidate) => {
		if (candidate.id !== candidateId) return candidate
		return {
			...candidate,
			artwork: {
				...candidate.artwork,
				proposed: candidate.artwork.proposed.filter(
					(asset) =>
						!(
							asset.kind === kind &&
							(asset.source === 'localFile' ||
								asset.source === 'steamGridDb' ||
								asset.source === 'missing')
						),
				),
			},
		}
	})
	state.invalidatePreview()
}
</script>

<template>
	<section class="grid gap-3">
		<div
			v-if="state.selectedCandidates.value.length === 0"
			class="grid min-h-55 place-items-center rounded-lg border border-dashed border-border-dashed bg-surface-3 p-6 text-secondary"
		>
			{{ t('artworkView.selectGamesPrompt') }}
		</div>

		<div v-else class="grid gap-3">
			<article
				v-for="candidate in state.selectedCandidates.value"
				:key="candidate.id"
				class="overflow-hidden rounded-lg border border-border bg-surface-3"
			>
				<header class="flex min-h-12 items-center gap-2 border-b border-border bg-surface-4 p-2">
					<GameIcon :candidate="candidate" :size="20" />
					<strong class="min-w-0 truncate text-base">{{ candidate.name }}</strong>
				</header>

				<div class="grid min-w-0 grid-cols-[repeat(auto-fit,minmax(190px,1fr))] gap-3 p-2">
					<div
						v-for="slot in slots"
						:key="slot.kind"
						class="grid min-w-0 grid-rows-[auto_auto_auto] gap-2 rounded-lg border border-border/60 bg-surface-5 p-2"
					>
						<div class="flex min-w-0 items-center justify-between gap-2">
							<strong class="shrink-0 text-sm">{{ slot.label }}</strong>
							<span
								class="min-w-0 truncate rounded-sm border border-border-muted px-1.5 py-0.5 text-xs text-secondary"
							>
								{{
									sourceLabel(
										selectedAsset(candidate, slot.kind) || existingAsset(candidate, slot.kind),
									)
								}}
							</span>
						</div>

						<div
							class="flex p-2 h-44 w-full items-center justify-center rounded-md border border-dashed border-border-dashed bg-surface-inset"
						>
							<img
								v-if="
									displayAsset(candidate, slot.kind)?.pathOrUrl &&
									!previewErrored(displayAsset(candidate, slot.kind))
								"
								class="max-h-full max-w-full object-contain"
								:src="previewSrc(displayAsset(candidate, slot.kind))"
								alt=""
								@error="markPreviewErrored(displayAsset(candidate, slot.kind))"
							/>
							<span v-else class="px-2 text-xs text-secondary">{{
								t('artworkSource.missing')
							}}</span>
						</div>

						<div class="flex gap-2">
							<UiButton
								class="h-9 flex-1"
								size="icon"
								variant="secondary"
								:title="t('artworkView.pickLocalArtworkTitle')"
								@click="pickArtwork(candidate.id, slot.kind)"
							>
								<FolderPlus :size="14" />
							</UiButton>
							<UiButton
								v-if="steamGridDbAvailable"
								class="h-9 flex-1"
								size="icon"
								variant="secondary"
								:title="t('artworkView.browseSteamGridDbTitle')"
								@click="openSteamGridDbBrowser(candidate, slot.kind)"
							>
								<SteamGridDbIcon class="h-3 w-auto" />
							</UiButton>
							<UiButton
								class="h-9 flex-1"
								size="icon"
								variant="ghost"
								:title="t('artworkView.useOfficialArtworkTitle')"
								:disabled="
									!candidate.artwork.proposed.some(
										(asset) => asset.kind === slot.kind && asset.source === 'officialSteam',
									)
								"
								@click="useOfficialArtwork(candidate.id, slot.kind)"
							>
								<RotateCcw :size="14" />
							</UiButton>
							<UiButton
								class="h-9 flex-1"
								size="icon"
								variant="danger"
								:title="t('artworkView.deleteArtworkTitle')"
								:disabled="!displayAsset(candidate, slot.kind)?.pathOrUrl"
								@click="deleteArtwork(candidate.id, slot.kind)"
							>
								<Trash2 :size="14" />
							</UiButton>
						</div>
					</div>
				</div>
			</article>
		</div>
	</section>

	<SteamGridDbBrowser
		v-if="browsingSlot"
		:api-key="state.settings.steamGridDb.apiKey ?? ''"
		:kind="browsingSlot.kind"
		:initial-query="browsingSlot.name"
		:allow-nsfw="state.settings.steamGridDb.allowNsfw"
		@close="browsingSlot = null"
		@select="onSteamGridDbSelect"
	/>
</template>
