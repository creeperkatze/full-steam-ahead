<script setup lang="ts">
import { AlertCircle, CheckCircle2, Clock, Loader2 } from '@lucide/vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import MarkdownIt from 'markdown-it'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import Modal from './Modal.vue'
import UiButton from './ui/Button.vue'

export type UpdateStatus = 'available' | 'downloading' | 'ready' | 'error'

const props = defineProps<{
	modelValue: boolean
	currentVersion: string
	latestVersion: string
	status: UpdateStatus
	progress: number
	errorMessage: string
	releaseNotes: string
}>()

const emit = defineEmits<{
	'update:modelValue': [value: boolean]
	update: []
	restart: []
}>()

const { t } = useI18n()

function close() {
	if (props.status === 'downloading') return
	emit('update:modelValue', false)
}

// Raw HTML in the notes is escaped and unsafe links are rejected
const markdown = new MarkdownIt({ breaks: true, linkify: true })

const renderedNotes = computed(() => markdown.render(props.releaseNotes.trim()))

function onNotesClick(e: MouseEvent) {
	const link = (e.target as HTMLElement).closest('a')
	if (!link) return
	e.preventDefault()
	openUrl(link.href)
}
</script>

<template>
	<Modal :model-value="modelValue" @update:model-value="close">
		<div class="mb-5 flex items-start gap-3">
			<Loader2
				v-if="status === 'downloading'"
				:size="20"
				class="mt-0.5 shrink-0 animate-spin text-accent"
			/>
			<CheckCircle2
				v-else-if="status === 'ready'"
				:size="20"
				class="mt-0.5 shrink-0 text-green-500"
			/>
			<AlertCircle v-else-if="status === 'error'" :size="20" class="mt-0.5 shrink-0 text-red-500" />
			<Clock v-else :size="20" class="mt-0.5 shrink-0 text-accent" />
			<div class="min-w-0">
				<h2 class="mb-1.5 text-base font-semibold">
					{{ status === 'ready' ? t('updateModal.readyTitle') : t('updateModal.title') }}
				</h2>
				<p class="text-sm text-secondary">
					<template v-if="status === 'error'">
						{{ t('updateModal.error', { message: errorMessage }) }}
					</template>
					<template v-else-if="status === 'downloading'">
						{{ t('updateModal.downloading') }}
					</template>
					<template v-else-if="status === 'ready'">
						{{ t('updateModal.readyDescription', { latest: latestVersion }) }}
					</template>
					<template v-else>
						{{ t('updateModal.description', { current: currentVersion, latest: latestVersion }) }}
					</template>
				</p>
				<!-- eslint-disable vue/no-v-html -- markdown-it escapes raw HTML -->
				<div
					v-if="status === 'available' && releaseNotes"
					class="release-notes mt-3 max-h-40 overflow-y-auto rounded-lg border border-border bg-surface-3 p-2.5 text-sm text-secondary"
					@click="onNotesClick"
					v-html="renderedNotes"
				/>
				<!-- eslint-enable vue/no-v-html -->
				<div
					v-if="status === 'downloading'"
					class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-surface-4"
				>
					<div
						class="h-full rounded-full bg-accent transition-all"
						:style="{ width: `${Math.round(progress * 100)}%` }"
					/>
				</div>
			</div>
		</div>
		<div class="flex justify-end gap-2">
			<UiButton v-if="status === 'ready'" variant="primary" @click="emit('restart')">
				{{ t('updateModal.restartNow') }}
			</UiButton>
			<template v-else>
				<UiButton variant="ghost" :disabled="status === 'downloading'" @click="close">
					{{ t('updateModal.later') }}
				</UiButton>
				<UiButton variant="primary" :disabled="status === 'downloading'" @click="emit('update')">
					<Loader2 v-if="status === 'downloading'" :size="14" class="animate-spin" />
					{{ status === 'error' ? t('updateModal.retry') : t('updateModal.update') }}
				</UiButton>
			</template>
		</div>
	</Modal>
</template>

<style>
.release-notes :is(h1, h2, h3, h4, p, ul, ol) {
	margin-top: 0.5rem;
}

.release-notes > :first-child {
	margin-top: 0;
}

.release-notes :is(h1, h2, h3, h4) {
	font-weight: 600;
	color: var(--color-primary);
}

.release-notes :is(ul, ol) {
	padding-left: 1rem;
}

.release-notes ul {
	list-style-type: disc;
}

.release-notes ol {
	list-style-type: decimal;
}

.release-notes strong {
	color: var(--color-primary);
}

.release-notes a {
	color: var(--color-accent);
	text-decoration: underline;
}

.release-notes code {
	border-radius: 0.25rem;
	background-color: var(--color-surface-4);
	padding: 0.05rem 0.3rem;
}
</style>
