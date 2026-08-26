<script setup lang="ts">
defineProps<{
	title: string
}>()
</script>

<template>
	<h2 class="mb-2 text-xs font-semibold tracking-wide text-secondary uppercase">{{ title }}</h2>
</template>
