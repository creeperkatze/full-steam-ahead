<script setup lang="ts">
import UiButton from "./ui/Button.vue";

defineProps<{
  backLabel?: string;
  nextLabel: string;
  nextDisabled?: boolean;
  nextVariant?: "primary" | "secondary" | "ghost" | "danger";
  showBack?: boolean;
}>();

defineEmits<{
  back: [];
  next: [];
}>();
</script>

<template>
  <footer class="fixed bottom-0 left-0 right-0 z-20 border-t border-border bg-surface-2/95 px-5 py-3 backdrop-blur">
    <div class="flex justify-end gap-2">
      <UiButton v-if="showBack" variant="ghost" @click="$emit('back')">
        {{ backLabel || "Back" }}
      </UiButton>
      <UiButton :variant="nextVariant || 'primary'" :disabled="nextDisabled" @click="$emit('next')">
        <slot name="next-icon" />
        {{ nextLabel }}
      </UiButton>
    </div>
  </footer>
</template>
