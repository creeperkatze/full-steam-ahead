<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{ modelValue: boolean; fullscreen?: boolean }>(), {
	fullscreen: false,
})
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

function onKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape' && props.modelValue) emit('update:modelValue', false)
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
	<Teleport to="body">
		<div
			v-if="modelValue"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
			@click="$emit('update:modelValue', false)"
		>
			<div
				class="rounded-xl border border-border bg-surface-2 shadow-xl"
				:class="
					fullscreen
						? 'flex h-[85vh] w-[90vw] max-w-5xl flex-col overflow-hidden'
						: 'mx-4 w-full max-w-sm p-6'
				"
				@click.stop
			>
				<slot />
			</div>
		</div>
	</Teleport>
</template>
