export interface Options {
  stopSteam: boolean;
  restartSteam: boolean;
  replaceExistingArtwork: boolean;
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
  gameName: string;
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
