<script setup lang="ts" generic="T extends { value: string; label: string }">
import { ChevronDown } from '@lucide/vue'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
	modelValue: string
	options: T[]
	disabled?: boolean
	placeholder?: string
}>()

const emit = defineEmits<{
	'update:modelValue': [value: string]
}>()

defineSlots<{
	leading?: (props: { option: T }) => unknown
}>()

const open = ref(false)
const containerRef = ref<HTMLElement>()
const focusedIndex = ref(-1)

const selectedOption = computed(() => props.options.find((o) => o.value === props.modelValue))

function select(option: T) {
	emit('update:modelValue', option.value)
	close()
}

function close() {
	open.value = false
	focusedIndex.value = -1
}

async function toggle() {
	if (props.disabled) return
	open.value = !open.value
	if (open.value) {
		focusedIndex.value = props.options.findIndex((o) => o.value === props.modelValue)
		await nextTick()
	}
}

function handleKeydown(e: KeyboardEvent) {
	if (!open.value) {
		if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
			e.preventDefault()
			toggle()
		}
		return
	}
	switch (e.key) {
		case 'Escape':
			close()
			break
		case 'ArrowDown':
			e.preventDefault()
			focusedIndex.value = Math.min(focusedIndex.value + 1, props.options.length - 1)
			break
		case 'ArrowUp':
			e.preventDefault()
			focusedIndex.value = Math.max(focusedIndex.value - 1, 0)
			break
		case 'Enter':
		case ' ':
			e.preventDefault()
			if (focusedIndex.value >= 0) select(props.options[focusedIndex.value])
			break
	}
}

function handleClickOutside(e: MouseEvent) {
	if (!containerRef.value?.contains(e.target as Node)) close()
}

watch(open, (isOpen) => {
	if (isOpen) document.addEventListener('mousedown', handleClickOutside)
	else document.removeEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => document.removeEventListener('mousedown', handleClickOutside))
</script>

<template>
	<div ref="containerRef" class="relative" @keydown="handleKeydown">
		<button
			type="button"
			:disabled="disabled"
			class="flex h-10 w-full min-w-0 items-center gap-2 rounded-md border border-border bg-surface-5 px-3 text-left text-sm transition-colors hover:bg-surface-4 disabled:cursor-not-allowed disabled:opacity-50"
			@click="toggle"
		>
			<slot v-if="selectedOption" name="leading" :option="selectedOption" />
			<span class="min-w-0 flex-1 truncate">
				{{ selectedOption?.label ?? placeholder ?? 'Select…' }}
			</span>
			<ChevronDown
				class="size-4 shrink-0 text-secondary transition-transform duration-150"
				:class="{ 'rotate-180': open }"
			/>
		</button>

		<Transition
			enter-active-class="transition-opacity duration-100"
			enter-from-class="opacity-0"
			leave-active-class="transition-opacity duration-75"
			leave-to-class="opacity-0"
		>
			<div
				v-if="open"
				class="absolute top-full z-50 mt-1 max-h-64 w-full min-w-max overflow-hidden overflow-y-auto rounded-md border border-border bg-surface-3 shadow-lg"
			>
				<button
					v-for="(option, index) in options"
					:key="option.value"
					type="button"
					class="flex w-full items-center gap-2 px-3 py-2.5 text-left text-sm text-primary transition-colors"
					:class="
						option.value === modelValue || focusedIndex === index
							? 'bg-surface-4'
							: 'hover:bg-surface-4'
					"
					@click="select(option)"
					@mouseenter="focusedIndex = index"
				>
					<slot name="leading" :option="option" />
					<span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
				</button>
			</div>
		</Transition>
	</div>
</template>
