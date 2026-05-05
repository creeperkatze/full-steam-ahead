<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "icon";
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    active?: boolean;
  }>(),
  {
    variant: "secondary",
    size: "md",
    type: "button",
    disabled: false,
    active: false
  }
);

const classes = computed(() => [
  "inline-flex items-center justify-center gap-2 rounded-md border text-fsa-text transition-colors disabled:cursor-not-allowed disabled:opacity-45",
  props.size === "icon" ? "h-9 w-9 p-0" : props.size === "sm" ? "h-8 px-3" : "h-9 px-3",
  props.variant === "primary" && "border-[#66c0f4] bg-[#1a9fff] font-bold text-[#07131f] hover:bg-[#66c0f4]",
  props.variant === "secondary" && "border-fsa-line bg-[#303844] hover:bg-[#394453]",
  props.variant === "ghost" && "border-fsa-line bg-transparent hover:bg-fsa-panel-2",
  props.variant === "danger" && "border-[#754037] bg-[#3a2421] text-[#f0a397] hover:bg-[#472b27]",
  props.active && "border-[#66c0f4] bg-[#12324a] text-fsa-text"
]);
</script>

<template>
  <button :type="type" :disabled="disabled" :class="classes">
    <slot />
  </button>
</template>
