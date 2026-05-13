<script setup lang="ts">
import Button from "./ui/Button.vue";

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
  <footer class="fixed bottom-5 left-0 right-0 z-20 flex justify-center px-5 pointer-events-none">
    <div class="pointer-events-auto flex items-center gap-2 rounded-xl border border-border bg-surface-2/95 px-4 py-2 shadow-lg backdrop-blur">
      <Button v-if="showBack" variant="ghost" @click="$emit('back')">
        {{ backLabel || "Back" }}
      </Button>
      <Button :variant="nextVariant || 'primary'" :disabled="nextDisabled" @click="$emit('next')">
        {{ nextLabel }}
        <template #icon>
          <slot name="next-icon" />
        </template>
      </Button>
    </div>
  </footer>
</template>
