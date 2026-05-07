import type { ApplyOptions } from "../types/steam";

const STORAGE_KEY = "full-steam-ahead:settings";

export const defaultApplyOptions: ApplyOptions = {
  stopSteam: false,
  restartSteam: false,
  replaceExistingArtwork: false,
  writeCollections: true,
  useLegacyCollectionsFallback: false
};

export interface AppSettings {
  options: ApplyOptions;
}

const defaultSettings: AppSettings = {
  options: defaultApplyOptions
};

export function loadSettings(): AppSettings {
  if (typeof window === "undefined") return cloneDefaultSettings();

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return cloneDefaultSettings();

    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return {
      options: {
        ...defaultApplyOptions,
        ...booleanOptions(parsed.options)
      }
    };
  } catch {
    return cloneDefaultSettings();
  }
}

export function saveSettings(settings: AppSettings) {
  if (typeof window === "undefined") return;

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

function cloneDefaultSettings(): AppSettings {
  return {
    ...defaultSettings,
    options: { ...defaultApplyOptions }
  };
}

function booleanOptions(value: unknown): Partial<ApplyOptions> {
  if (!value || typeof value !== "object") return {};

  const parsed = value as Partial<Record<keyof ApplyOptions, unknown>>;
  return {
    stopSteam: booleanOr(parsed.stopSteam, defaultApplyOptions.stopSteam),
    restartSteam: booleanOr(parsed.restartSteam, defaultApplyOptions.restartSteam),
    replaceExistingArtwork: booleanOr(parsed.replaceExistingArtwork, defaultApplyOptions.replaceExistingArtwork),
    writeCollections: booleanOr(parsed.writeCollections, defaultApplyOptions.writeCollections),
    useLegacyCollectionsFallback: booleanOr(
      parsed.useLegacyCollectionsFallback,
      defaultApplyOptions.useLegacyCollectionsFallback
    )
  };
}

function booleanOr(value: unknown, fallback: boolean) {
  return typeof value === "boolean" ? value : fallback;
}
