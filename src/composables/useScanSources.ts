import { listen } from "@tauri-apps/api/event";
import { ref } from "vue";
import { api } from "../helpers/api";
import { importSourceName } from "../helpers/sourceNames";
import type { ImportCandidate, ScanProgressEvent } from "../types";
import { useAppState } from "./useAppState";
import { useTaskStatus } from "./useTaskStatus";

export type ScannableSource =
  | "playnite"
  | "epic"
  | "amazon"
  | "gog"
  | "itch"
  | "origin"
  | "ubisoftConnect"
  | "gamePass";

export interface SourceState {
  key: ScannableSource;
  name: string;
  status: "pending" | "scanning" | "done";
  found: number;
}

export const SCANNABLE_SOURCES: ScannableSource[] = [
  "playnite",
  "epic",
  "amazon",
  "gog",
  "itch",
  "origin",
  "ubisoftConnect",
  "gamePass"
];

const sourceStates = ref<SourceState[]>(makeSourceStates());
let unlistenScan: (() => void) | undefined;

function makeSourceStates(): SourceState[] {
  return SCANNABLE_SOURCES.map(key => ({
    key,
    name: importSourceName(key),
    status: "pending" as const,
    found: 0
  }));
}

function mergeCandidates(existing: ImportCandidate[], incoming: ImportCandidate[]) {
  const map = new Map(existing.map(c => [c.id, c]));
  for (const candidate of incoming) {
    map.set(candidate.id, candidate);
  }
  return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name));
}

export function useScanSources() {
  const state = useAppState();
  const task = useTaskStatus();

  async function scan() {
    if (!state.selectedUserId.value) return;

    sourceStates.value = makeSourceStates();
    state.scanPhase.value = "scanning";

    unlistenScan?.();
    unlistenScan = await listen<ScanProgressEvent>("scan-progress", (event) => {
      const { source, status, found } = event.payload;
      const entry = sourceStates.value.find(s => s.name === source);
      if (entry) {
        if (status === "scanning") {
          entry.status = "scanning";
        } else if (status === "done") {
          entry.status = "done";
          entry.found = found;
        }
      }
    });

    const found = await task.runTask("Scanning sources", () =>
      api.scanSources({ userSteamId: state.selectedUserId.value, includeSources: [] })
    );

    unlistenScan();
    unlistenScan = undefined;

    if (found !== undefined) {
      state.candidates.value = mergeCandidates(state.candidates.value, found);
      state.selectedCandidateIds.value = new Set(state.candidates.value.map(c => c.id));
      state.invalidatePreview();
      state.scanPhase.value = "done";
    } else {
      state.scanPhase.value = "idle";
    }
  }

  return { sourceStates, scan };
}
