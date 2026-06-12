<script setup lang="ts">
import { User } from '@lucide/vue'
import { ref, watch } from 'vue'

const props = defineProps<{
	src: string | null | undefined
	size?: number
}>()

const errored = ref(false)

watch(
	() => props.src,
	() => {
		errored.value = false
	},
)
</script>

<template>
	<img
		v-if="src && !errored"
		:src="src"
		class="shrink-0 rounded-full object-cover"
		:style="{ width: `${size ?? 18}px`, height: `${size ?? 18}px` }"
		alt=""
		@error="errored = true"
	/>
	<User v-else :size="size ?? 18" class="shrink-0 text-secondary" />
</template>
