import { invoke } from "@tauri-apps/api/core";
import type {
  ApplyOptions,
  ApplyResult,
  ImportCandidate,
  ManualImportRequest,
  PreviewPlan,
  ScanRequest,
  ShortcutEntry,
  SteamInstallation
} from "../types/steam";

export const api = {
  detectSteam: () => invoke<SteamInstallation>("detect_steam"),
  readShortcuts: (userSteamId: string) =>
    invoke<ShortcutEntry[]>("read_shortcuts_for_user", { userSteamId }),
  scanSources: (request: ScanRequest) => invoke<ImportCandidate[]>("scan_sources", { request }),
  createManualCandidate: (request: ManualImportRequest) =>
    invoke<ImportCandidate>("create_manual_candidate", { request }),
  createPreviewPlan: (
    userSteamId: string,
    candidates: ImportCandidate[],
    options: ApplyOptions
  ) => invoke<PreviewPlan>("create_preview_plan", { userSteamId, candidates, options }),
  applyPlan: (plan: PreviewPlan, candidates: ImportCandidate[], options: ApplyOptions) =>
    invoke<ApplyResult>("apply_plan", { request: { plan, candidates, options } })
};
