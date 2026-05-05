import type { ImportSource } from "./types";

const IMPORT_SOURCE_NAMES: Record<Exclude<ImportSource, { other: string }>, string> = {
  manual: "Manual",
  playnite: "Playnite",
  epic: "Epic Games",
  gog: "GOG"
};

export function importSourceName(source: ImportSource) {
  return typeof source === "string" ? IMPORT_SOURCE_NAMES[source] : source.other;
}
