export type ImportSource =
  | "manual"
  | "playnite"
  | "epic"
  | "gog"
  | "amazon"
  | "bottles"
  | "flatpak"
  | "gamePass"
  | "heroic"
  | "itch"
  | "legendary"
  | "lutris"
  | "miniGalaxy"
  | "origin"
  | "ubisoftConnect"
  | { other: string };

export interface ImportCandidate {
  id: string;
  source: ImportSource;
  name: string;
  executablePath: string;
  startDir: string;
  launchOptions?: string | null;
  existingAppId?: number | null;
  matchedSteamAppId?: number | null;
  tags: string[];
  artwork: ArtworkPlan;
}

export interface ManualImportRequest {
  userSteamId: string;
  executablePath: string;
  displayName?: string;
  source: ImportSource;
  tags: string[];
}

export interface ScanRequest {
  userSteamId: string;
  includeSources: ImportSource[];
}

export interface ArtworkPlan {
  mode: "preserveExisting" | "officialSteamPreferred" | "steamGridDbFallback" | "localOverride";
  existing: ArtworkAsset[];
  proposed: ArtworkAsset[];
}

export type ArtworkKind = "header" | "capsule" | "hero" | "logo" | "icon";

export interface ArtworkAsset {
  kind: ArtworkKind;
  pathOrUrl: string;
  source: "existingCustom" | "officialSteam" | "steamGridDb" | "localFile";
  willReplaceExisting: boolean;
}
