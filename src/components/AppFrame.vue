<script setup lang="ts">
import { Settings } from "@lucide/vue";
import UiButton from "./ui/Button.vue";
import appIconUrl from "../assets/icon.svg";

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
  <main class="grid min-h-screen grid-rows-[76px_1fr] bg-fsa-bg text-fsa-text">
    <header class="grid grid-cols-[280px_1fr_auto] items-center gap-5 border-b border-fsa-line bg-fsa-header px-5">
      <div class="flex items-center gap-3">
        <img class="size-8 rounded-md" :src="appIconUrl" alt="" />
        <div>
          <strong class="block">Full Steam Ahead</strong>
          <span class="block text-fsa-muted">Import safely, review everything.</span>
        </div>
      </div>

      <nav v-if="!settingsOpen" class="grid grid-cols-3 gap-2" aria-label="Import progress">
        <span
          v-for="(step, index) in steps"
          :key="step"
          class="flex min-h-9 items-center gap-2 rounded-md border px-3 text-fsa-muted"
          :class="
            activeStep >= index
              ? 'border-[#66c0f4] bg-[#12324a] text-fsa-text'
              : 'border-fsa-line bg-fsa-panel-3'
          "
        >
          <b
            class="grid size-5 place-items-center rounded-full text-xs"
            :class="activeStep >= index ? 'bg-fsa-accent text-[#07131f]' : 'bg-[#3a424d] text-fsa-muted'"
          >
            {{ index + 1 }}
          </b>
          {{ step }}
        </span>
      </nav>

      <div v-else class="rounded-md border border-[#66c0f4] bg-[#12324a] px-4 py-2">
        <strong class="block">Settings</strong>
        <span class="text-fsa-muted">Application preferences and apply behavior</span>
      </div>

      <div class="flex items-center gap-2">
        <UiButton size="icon" variant="ghost" title="Settings" :active="settingsOpen" @click="$emit('toggle-settings')">
          <Settings :size="17" />
        </UiButton>
      </div>
    </header>

    <section class="min-w-0 px-5 pb-24 pt-4">
      <div v-if="error" class="mb-3 rounded-md border border-[#754037] bg-[#2b1d1a] px-3 py-2 text-[#f0a397]">
        {{ error }}
      </div>

      <slot />
    </section>
  </main>
</template>
