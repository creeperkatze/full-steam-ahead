# Importers

An importer finds one launcher's installed games and returns them as `ImportCandidate`s. Each has a `scan(user, custom_path)` function in `src-tauri/src/importers/`. `custom_path` is the folder the user set for that source in the settings, if any.

## Adding an importer

1. Add a variant to `ImportSource` in `models/importers.rs`, with a `display_name` and a `settings_key`.
2. Add the module to `importers/mod.rs`. Put it behind `#[cfg(windows)]` or `#[cfg(unix)]` if the launcher only exists on one platform.
3. Register `scan` in `importer_registry` in `importers/scan.rs`. The registry also decides which sources the settings show.
4. Add the name to `src/helpers/sourceNames.ts` and the icon to `src/components/SourceIcon.vue`, then run `pnpm gen:bindings`.

Leave `existing_app_id` alone. The scan links every candidate to the shortcut it already has in Steam (see [STEAM.md](STEAM.md)).

## Where the data comes from

In order of preference:

1. **The launcher's own CLI**, when it can list installed games. Lutris, Legendary, Bottles and Flatpak do this, and Xbox uses `Get-AppxPackage` through PowerShell. Run it with `command_stdout` or `command_output`.
2. **The launcher's own files**, read with a real parser: JSON with serde (`read_launcher_json`, `parse_launcher_json`), SQLite with the `sqlite` crate, XML with `roxmltree`, LiteDB with `util::litedb`.
3. **The Windows registry**, through the `Registry` trait in `util::registry`. On Linux, `wine::wine_registries` reads the same keys from a Wine or Proton prefix's `system.reg`, so Windows launchers under Proton share one code path.

The helpers in `importers/mod.rs` log for you. A missing file or program is expected and logged at `debug`. Anything else is a `warn` with a snippet of the output.

Never scan raw bytes for marker strings or cut text at guessed delimiters. If no library fits, write a small, tested reader that follows the format's documented layout, like `util::litedb`.

Launchers change their formats. Check the upstream source of open-source launchers before trusting existing code.

## Launching

A candidate starts either through its launcher or through the game's own executable.

- **Launcher only**: build it with `launcher_candidate` and the launcher's URL or command (`heroic://launch/…`, `uplay://…`, `origin2://…`).
- **Both routes**: build it with `candidate_from_parts` for the executable. Then set `url_scheme` and `launcher_path` from `launcher_url_pair`, and set `use_launcher_url` to the importer's default. The user's launcher setting can override that default (`apply_launcher_mode`). See `epic.rs`.
- **Executable only**: `candidate_from_parts`.

Default to the launcher when the game can't start correctly without it, for example when it needs auth arguments or DLC that only the launcher passes.

On Linux:

- Host binaries are reached through `sandbox::host_command` (for running a CLI) and `sandbox::host_launch` (for the shortcut). They handle FSA or Steam running as a Flatpak. `launcher_candidate` and `launcher_url_pair` already call `host_launch`.
- A game found inside a Proton prefix gets its launch options from `wine::proton_launch_options` and sets `needs_proton`, so the import maps Proton for it.
- Quote values that go into launch options with `shell_quote`. Steam shell-expands them.

## Scope

- Skip entries that can't be launched (uninstalled, DLC, missing executable or folder) and log why at `debug`.
- Don't take shortcut icons from launcher or install metadata. Artwork comes from the artwork pipeline.

## Checking a change

- Unit-test parsers against real sample data. Small fixtures written by the real tool are best. Keep the generator next to the fixture, like `util/fixtures/litedb_v4.cs`.
- Windows-only importers can't be compiled on Linux or macOS, and Unix-only ones can't be compiled on Windows. Say so when a change couldn't be compiled locally, and rely on CI.
