import type { ImportSource } from "../types";

const IMPORT_SOURCE_NAMES: Record<Exclude<ImportSource, { other: string }>, string> = {
  manual: "Manual",
  playnite: "Playnite",
  epic: "Epic Games",
  gog: "GOG",
  amazon: "Amazon Games",
  bottles: "Bottles",
  flatpak: "Flatpak",
  gamePass: "Game Pass",
  heroic: "Heroic",
  itch: "itch.io",
  legendary: "Legendary",
  lutris: "Lutris",
  miniGalaxy: "MiniGalaxy",
  origin: "EA app / Origin",
  ubisoftConnect: "Ubisoft Connect"
};

export function importSourceName(source: ImportSource) {
  return typeof source === "string" ? IMPORT_SOURCE_NAMES[source] : source.other;
}
