<script setup lang="ts">
import { ArrowRight, Check } from "@lucide/vue";
import BottomActionBar from "../../../components/BottomActionBar.vue";
import ArtworkView from "./ArtworkView.vue";
import ReviewView from "./ReviewView.vue";
import SourcesView from "./SourcesView.vue";
import { useAppState } from "../../../composables/useAppState";

const state = useAppState();
</script>

<template>
  <div class="grid content-start gap-3">
    <SourcesView
      v-if="state.step.value === 'sources'"
      v-model:manual-path="state.manualPath.value"
      v-model:manual-name="state.manualName.value"
      v-model:include-playnite="state.includePlaynite.value"
      v-model:include-epic="state.includeEpic.value"
      :install="state.install.value"
      v-model:selected-user-id="state.selectedUserId.value"
      :selected-user="state.selectedUser.value"
      :candidates="state.candidates.value"
      :selected-ids="state.selectedCandidateIds.value"
      :loading="state.loading.value"
      @refresh-steam="state.refreshSteam"
      @scan="state.scan"
      @pick-executable="state.pickExecutable"
      @add-manual="state.addManual"
      @toggle-candidate="state.toggleCandidate"
      @select-all="state.selectAll"
      @select-none="state.selectNone"
    />

    <ArtworkView
      v-else-if="state.step.value === 'artwork'"
      v-model:replace-existing-artwork="state.options.value.replaceExistingArtwork"
      :candidates="state.selectedCandidates.value"
      :custom-artwork="state.customArtwork.value"
      @pick-artwork="state.pickArtwork"
      @use-official-artwork="state.useOfficialArtwork"
    />

    <ReviewView
      v-else
      :plan="state.previewPlan.value"
      :apply-result="state.applyResult.value"
    />

    <BottomActionBar
      :show-back="state.step.value !== 'sources'"
      :next-label="state.nextLabel.value"
      :next-disabled="state.nextDisabled.value"
      @back="state.goBack"
      @next="state.goNext"
    >
      <template #next-icon>
        <Check v-if="state.step.value === 'review'" :size="18" />
        <ArrowRight v-else :size="16" />
      </template>
    </BottomActionBar>
  </div>
</template>
