# Importers

Each importer finds one launcher's installed games and turns them into import candidates.

## Where the data comes from

In order of preference:

1. **The launcher's own CLI**, when it can list installed games (Lutris, Legendary, Bottles, Flatpak). Use it and nothing else.
2. **The launcher's own data**, read with a real parser: JSON with serde, SQLite with the `sqlite` crate, XML with `roxmltree`, LiteDB with `util::litedb`.
3. **The Windows registry**, through the `Registry` trait in `util::registry`. On Linux the same code reads a Wine/Proton prefix's `system.reg`, so Windows launchers running under Proton share one code path.

Never scan raw bytes for marker strings or cut text at guessed delimiters. If no library fits, write a small, tested reader that follows the format's documented layout (see `util::litedb`).

## Launching

Prefer the launcher's own launch URL or command (`heroic://launch/<runner>/<id>`, `uplay://…`, `origin2://…`) over pointing Steam at the game executable. Under Proton, launch options go through `proton_launch_options`.

## Scope

- Don't take shortcut icons from launcher or install metadata; artwork comes from the artwork pipeline.
- Don't port features from BoilR that the existing selection UI already covers (blacklists, per-game toggles).
- Skip entries that can't be launched (uninstalled, DLC, missing executable) and log why at `debug`.

## Checking a change

- Unit-test parsers against real sample data. Small fixtures written by the real tool are best; keep the generator next to the fixture.
- Linux-only importers can't be compiled on Windows. Say so, and rely on CI.
