<script setup lang="ts">
import { AlertTriangle, ShieldCheck } from "@lucide/vue";
import type { ApplyResult, PreviewPlan } from "../types/steam";

defineProps<{
  plan: PreviewPlan | null;
  applyResult: ApplyResult | null;
}>();
</script>

<template>
  <section class="rounded-lg border border-border bg-surface-3 p-4">
    <div class="mb-3">
      <h1 class="text-[26px] font-bold leading-tight">Final Review</h1>
      <p class="text-secondary">These are the exact files and changes that will be touched.</p>
    </div>

    <div class="mb-3 flex min-h-10 items-center gap-2 rounded-md border border-border bg-surface-5 px-3 py-2">
      <ShieldCheck :size="18" />
      <span>Backups are created before any Steam file is written.</span>
    </div>

    <div
      v-if="plan?.warnings.length"
      class="mb-3 flex min-h-10 items-center gap-2 rounded-md border border-warning-border bg-warning-bg px-3 py-2 text-warning"
    >
      <AlertTriangle :size="18" />
      <span>{{ plan.warnings.join(" ") }}</span>
    </div>

    <div class="overflow-hidden rounded-md border border-border">
      <table>
        <thead class="bg-surface-4">
          <tr>
            <th>Change</th>
            <th>File</th>
            <th>Risk</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="change in plan?.changes ?? []" :key="change.id">
            <td>
              <strong>{{ change.title }}</strong>
              <small>{{ change.details }}</small>
            </td>
            <td class="path-cell">{{ change.file }}</td>
            <td>
              <span
                class="inline-flex rounded-full border px-2 py-0.5"
                :class="change.destructive ? 'border-danger-border-muted text-danger' : 'border-accent text-accent-text'"
              >
                {{ change.destructive ? "Destructive" : "Additive" }}
              </span>
            </td>
          </tr>
          <tr v-if="!plan">
            <td colspan="3" class="h-20 text-center text-secondary">Preparing preview...</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="mt-3 grid grid-cols-[minmax(0,1fr)_260px] gap-3">
      <div class="min-w-0 rounded-md border border-border bg-surface-5 p-3">
        <h2 class="mb-2 text-base font-semibold">Backups</h2>
        <p v-for="backup in plan?.backups ?? []" :key="backup.destination" class="path-cell">
          {{ backup.source }} -> {{ backup.destination }}
        </p>
      </div>
      <div v-if="applyResult" class="rounded-md border border-border bg-surface-5 p-3">
        <h2 class="mb-2 text-base font-semibold">Applied</h2>
        <p>{{ applyResult.appliedChanges.length }} changes applied.</p>
        <p>{{ applyResult.backupsCreated.length }} backups created.</p>
      </div>
    </div>
  </section>
</template>
