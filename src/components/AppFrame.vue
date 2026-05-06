<script setup lang="ts">
import { Settings } from "@lucide/vue";
import UiButton from "./ui/Button.vue";
import Logo from "../assets/logo.svg?component";

defineProps<{
  activeStep: number;
  error: string;
  settingsOpen: boolean;
}>();

defineEmits<{
  "toggle-settings": [];
}>();

const steps = ["Sources", "Artwork", "Review"];
</script>

<template>
  <main class="grid min-h-screen grid-rows-[76px_1fr] bg-surface-1 text-primary">
    <header class="grid grid-cols-[280px_1fr_auto] items-center gap-5 border-b border-border bg-surface-2 px-2">
      <div class="flex items-center">
        <Logo class="h-12 w-auto" aria-label="Full Steam Ahead" />
      </div>

      <nav v-if="!settingsOpen" class="grid grid-cols-3 gap-2" aria-label="Import progress">
        <span
          v-for="(step, index) in steps"
          :key="step"
          class="flex min-h-9 items-center gap-2 rounded-md border px-3 text-secondary"
          :class="
            activeStep >= index
              ? 'border-accent bg-accent-bg text-primary'
              : 'border-border bg-surface-5'
          "
        >
          <b
            class="grid size-5 place-items-center rounded-full text-xs"
            :class="activeStep >= index ? 'bg-accent text-accent-contrast' : 'bg-border-muted text-secondary'"
          >
            {{ index + 1 }}
          </b>
          {{ step }}
        </span>
      </nav>

      <div v-else class="rounded-md border border-accent bg-accent-bg px-4 py-2">
        <strong class="block">Settings</strong>
        <span class="text-secondary">Application preferences and apply behavior</span>
      </div>

      <div class="flex items-center gap-2">
        <UiButton size="icon" variant="ghost" title="Settings" :active="settingsOpen" @click="$emit('toggle-settings')">
          <Settings :size="17" />
        </UiButton>
      </div>
    </header>

    <section class="min-w-0 px-5 pb-24 pt-4">
      <div v-if="error" class="mb-3 rounded-md border border-danger-border bg-danger-bg px-3 py-2 text-danger">
        {{ error }}
      </div>

      <slot />
    </section>
  </main>
</template>
