import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api/steam";
import type {
  ApplyOptions,
  ApplyResult,
  ArtworkAsset,
  ArtworkKind,
  ImportCandidate,
  PreviewPlan,
  SteamInstallation,
  SteamUser
} from "../types/steam";

export type FlowStep = "sources" | "artwork" | "review";

const step = ref<FlowStep>("sources");
const install = ref<SteamInstallation | null>(null);
const selectedUserId = ref("");
const candidates = ref<ImportCandidate[]>([]);
const selectedCandidateIds = ref<Set<string>>(new Set());
const previewPlan = ref<PreviewPlan | null>(null);
const applyResult = ref<ApplyResult | null>(null);
const customArtwork = ref<Record<string, string>>({});
const loading = ref(false);
const error = ref("");
const status = ref("Ready");
const manualPath = ref("");
const manualName = ref("");
const includePlaynite = ref(true);
const includeEpic = ref(true);
const initialized = ref(false);
const options = ref<ApplyOptions>({
  stopSteam: false,
  restartSteam: false,
  replaceExistingArtwork: false,
  writeCollections: true,
  useLegacyCollectionsFallback: false
});

watch(
  options,
  () => {
    previewPlan.value = null;
    applyResult.value = null;
    if (step.value === "review" && selectedCandidates.value.length > 0) {
      void continueToReview();
    }
  },
  { deep: true }
);

const selectedUser = computed<SteamUser | undefined>(() =>
  install.value?.users.find((user) => user.steamId === selectedUserId.value)
);

const selectedCandidates = computed(() =>
  candidates.value.filter((candidate) => selectedCandidateIds.value.has(candidate.id))
);

const activeStepIndex = computed(() => {
  if (step.value === "artwork") return 1;
  if (step.value === "review") return 2;
  return 0;
});

const nextLabel = computed(() => {
  if (step.value === "review") return "Apply";
  return "Continue";
});

const nextDisabled = computed(() => {
  if (loading.value) return true;
  if (step.value === "sources") return selectedCandidates.value.length === 0;
  if (step.value === "review") return !previewPlan.value;
  return false;
});

async function initialize() {
  if (initialized.value) return;
  initialized.value = true;
  await refreshSteam();
}

async function runTask<T>(label: string, task: () => Promise<T>): Promise<T | undefined> {
  loading.value = true;
  error.value = "";
  status.value = label;
  try {
    return await task();
  } catch (err) {
    error.value = commandMessage(err);
  } finally {
    loading.value = false;
    status.value = "Ready";
  }
}

async function refreshSteam() {
  const detected = await runTask("Detecting Steam", () => api.detectSteam());
  if (!detected) return;
  install.value = detected;
  selectedUserId.value = detected.users[0]?.steamId ?? "";
  previewPlan.value = null;
  if (selectedUserId.value) {
    await scan();
  }
}

async function scan() {
  if (!selectedUserId.value) return;
  const found = await runTask("Scanning sources", () =>
    api.scanSources({
      userSteamId: selectedUserId.value,
      includePlaynite: includePlaynite.value,
      includeEpic: includeEpic.value
    })
  );
  if (!found) return;
  candidates.value = mergeCandidates(candidates.value, found);
  selectedCandidateIds.value = new Set(candidates.value.map((candidate) => candidate.id));
  previewPlan.value = null;
}

async function pickExecutable() {
  const picked = await open({
    multiple: false,
    filters: [{ name: "Executable", extensions: ["exe", "bat", "cmd"] }]
  });
  if (typeof picked === "string") {
    manualPath.value = picked;
  }
}

async function addManual() {
  if (!selectedUserId.value || !manualPath.value.trim()) return;
  const candidate = await runTask("Adding manual entry", () =>
    api.createManualCandidate({
      userSteamId: selectedUserId.value,
      executablePath: manualPath.value.trim(),
      displayName: manualName.value.trim() || undefined,
      source: "manual",
      tags: ["Manual"]
    })
  );
  if (!candidate) return;
  candidates.value = mergeCandidates(candidates.value, [candidate]);
  selectedCandidateIds.value = new Set([...selectedCandidateIds.value, candidate.id]);
  manualPath.value = "";
  manualName.value = "";
  previewPlan.value = null;
}

async function pickArtwork(candidateId: string, kind: ArtworkKind) {
  const picked = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }]
  });
  if (typeof picked !== "string") return;
  const key = artworkKey(candidateId, kind);
  customArtwork.value = { ...customArtwork.value, [key]: picked };
  upsertArtworkAsset(candidateId, {
    kind,
    pathOrUrl: picked,
    source: "localFile",
    willReplaceExisting: true
  });
}

function useOfficialArtwork(candidateId: string, kind: ArtworkKind) {
  const candidate = candidates.value.find((candidate) => candidate.id === candidateId);
  const official = candidate?.artwork.proposed.find(
    (asset) => asset.kind === kind && asset.source === "officialSteam"
  );
  if (!official) return;
  removeLocalArtworkAsset(candidateId, kind);
  const { [artworkKey(candidateId, kind)]: _removed, ...rest } = customArtwork.value;
  customArtwork.value = rest;
}

function upsertArtworkAsset(candidateId: string, asset: ArtworkAsset) {
  candidates.value = candidates.value.map((candidate) => {
    if (candidate.id !== candidateId) return candidate;
    const proposed = candidate.artwork.proposed.filter(
      (item) => !(item.kind === asset.kind && item.source === asset.source)
    );
    return {
      ...candidate,
      artwork: {
        ...candidate.artwork,
        mode: asset.source === "localFile" ? "localOverride" : candidate.artwork.mode,
        proposed: [...proposed, asset]
      }
    };
  });
  previewPlan.value = null;
}

function removeLocalArtworkAsset(candidateId: string, kind: ArtworkKind) {
  candidates.value = candidates.value.map((candidate) => {
    if (candidate.id !== candidateId) return candidate;
    return {
      ...candidate,
      artwork: {
        ...candidate.artwork,
        proposed: candidate.artwork.proposed.filter(
          (asset) => !(asset.kind === kind && asset.source === "localFile")
        )
      }
    };
  });
  previewPlan.value = null;
}

function artworkKey(candidateId: string, kind: ArtworkKind) {
  return `${candidateId}:${kind}`;
}

async function continueToReview() {
  if (!selectedUserId.value) return;
  const plan = await runTask("Creating preview", () =>
    api.createPreviewPlan(selectedUserId.value, selectedCandidates.value, options.value)
  );
  if (!plan) return;
  previewPlan.value = plan;
  applyResult.value = null;
  step.value = "review";
}

async function apply() {
  if (!previewPlan.value) return;
  const result = await runTask("Applying changes", () =>
    api.applyPlan(previewPlan.value as PreviewPlan, selectedCandidates.value, options.value)
  );
  if (result) {
    applyResult.value = result;
  }
}

function goBack() {
  if (step.value === "review") {
    step.value = "artwork";
  } else if (step.value === "artwork") {
    step.value = "sources";
  }
}

async function goNext() {
  if (step.value === "sources") {
    step.value = "artwork";
    return;
  }

  if (step.value === "artwork") {
    await continueToReview();
    return;
  }

  await apply();
}

async function goToStepIndex(index: number) {
  if (index === 0) {
    step.value = "sources";
    return;
  }

  if (index === 1) {
    if (selectedCandidates.value.length > 0) {
      step.value = "artwork";
    }
    return;
  }

  if (index === 2 && selectedCandidates.value.length > 0) {
    if (previewPlan.value) {
      step.value = "review";
      return;
    }
    await continueToReview();
  }
}

function toggleCandidate(id: string) {
  const next = new Set(selectedCandidateIds.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  selectedCandidateIds.value = next;
  previewPlan.value = null;
}

function selectAll() {
  selectedCandidateIds.value = new Set(candidates.value.map((candidate) => candidate.id));
  previewPlan.value = null;
}

function selectNone() {
  selectedCandidateIds.value = new Set();
  previewPlan.value = null;
}

function mergeCandidates(existing: ImportCandidate[], incoming: ImportCandidate[]) {
  const map = new Map(existing.map((candidate) => [candidate.id, candidate]));
  for (const candidate of incoming) {
    map.set(candidate.id, candidate);
  }
  return Array.from(map.values()).sort((left, right) => left.name.localeCompare(right.name));
}

function commandMessage(err: unknown) {
  if (typeof err === "object" && err && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
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
    loading,
    error,
    status,
    manualPath,
    manualName,
    includePlaynite,
    includeEpic,
    options,
    selectedUser,
    selectedCandidates,
    activeStepIndex,
    nextLabel,
    nextDisabled,
    initialize,
    refreshSteam,
    scan,
    pickExecutable,
    addManual,
    pickArtwork,
    useOfficialArtwork,
    artworkKey,
    goBack,
    goNext,
    goToStepIndex,
    toggleCandidate,
    selectAll,
    selectNone
  };
}
