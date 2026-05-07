import { computed, ref, watch } from "vue";
import { loadSettings, saveSettings } from "../helpers/settings";
import type {
  ApplyOptions,
  ApplyResult,
  ImportCandidate,
  PreviewPlan,
  SteamInstallation,
  SteamUser
} from "../types/steam";

export type FlowStep = "sources" | "artwork" | "review";

const savedSettings = loadSettings();

const step = ref<FlowStep>("sources");
const install = ref<SteamInstallation | null>(null);
const selectedUserId = ref("");
const candidates = ref<ImportCandidate[]>([]);
const selectedCandidateIds = ref<Set<string>>(new Set());
const previewPlan = ref<PreviewPlan | null>(null);
const applyResult = ref<ApplyResult | null>(null);
const customArtwork = ref<Record<string, string>>({});
const manualPath = ref("");
const manualName = ref("");
const includePlaynite = ref(savedSettings.includePlaynite);
const includeEpic = ref(savedSettings.includeEpic);
const options = ref<ApplyOptions>(savedSettings.options);

watch(
  options,
  () => {
    persistSettings();
    invalidatePreview();
  },
  { deep: true }
);

watch([includePlaynite, includeEpic], () => {
  persistSettings();
});

const selectedUser = computed<SteamUser | undefined>(() =>
  install.value?.users.find((user) => user.steamId === selectedUserId.value)
);

const selectedCandidates = computed(() =>
  candidates.value.filter((candidate) => selectedCandidateIds.value.has(candidate.id))
);

function invalidatePreview() {
  previewPlan.value = null;
  applyResult.value = null;
}

function persistSettings() {
  saveSettings({
    includePlaynite: includePlaynite.value,
    includeEpic: includeEpic.value,
    options: options.value
  });
}

export function useAppState() {
  return {
    step,
    install,
    selectedUserId,
    candidates,
    selectedCandidateIds,
    previewPlan,
    applyResult,
    customArtwork,
    manualPath,
    manualName,
    includePlaynite,
    includeEpic,
    options,
    selectedUser,
    selectedCandidates,
    invalidatePreview
  };
}
