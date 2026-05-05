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

export interface SteamInstallation {
  installPath: string;
  users: SteamUser[];
  running: boolean;
}

export interface SteamUser {
  steamId: string;
  accountName?: string | null;
  shortcutsPath: string;
  gridPath: string;
  collectionsPath: string;
}

export interface ShortcutEntry {
  appId: number;
  appName: string;
  exe: string;
  startDir: string;
  icon: string;
  shortcutPath: string;
  launchOptions: string;
  isHidden: boolean;
  allowDesktopConfig: boolean;
  allowOverlay: boolean;
  openVr: boolean;
  devkit: boolean;
  devkitGameId: string;
  lastPlayTime: number;
  tags: string[];
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
  includePlaynite: boolean;
  includeEpic: boolean;
}

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

export interface ApplyOptions {
  stopSteam: boolean;
  restartSteam: boolean;
  replaceExistingArtwork: boolean;
  writeCollections: boolean;
  useLegacyCollectionsFallback: boolean;
}

export interface PreviewPlan {
  userSteamId: string;
  changes: PlannedChange[];
  filesToChange: string[];
  backups: BackupPlan[];
  warnings: string[];
  requiresSteamRestart: boolean;
}

export interface BackupPlan {
  source: string;
  destination: string;
}

export interface PlannedChange {
  id: string;
  title: string;
  file: string;
  kind: "addShortcut" | "updateShortcut" | "writeArtwork" | "updateCollections";
  destructive: boolean;
  details: string;
}

export interface ApplyResult {
  appliedChanges: PlannedChange[];
  backupsCreated: string[];
  skippedChanges: string[];
}
