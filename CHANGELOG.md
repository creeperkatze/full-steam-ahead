# Changelog

## [Unreleased]

### Security

- Updated the self updater to include the latest security fixes from Tauri.

## [1.6.2] - 2026-10-25

### Added

- Games from the new Epic Games Launcher are now found on Windows. The old launcher is still supported.
- Italian translation.

## [1.6.1] - 2026-10-14

### Fixed

- Fixed some source icons not showing on Linux.

## [1.6.0] - 2026-10-11

### Added

- Games from Rare are now found when Rare is installed as a Flatpak.
- The artwork and review steps now show which launcher each game comes from.

### Changed

- Artwork a game already has in Steam is now always kept. Use the "Use official Steam artwork" button to replace it on purpose.
- A game keeps its Steam entry when you rename it or switch between launching through the launcher and the game itself. Games you renamed in an older version are imported once more as a new entry, so delete the old shortcut in Steam.
- Games are now imported whenever their launcher lists them as installed, without also checking their install folder.
- Artwork left behind by a shortcut you deleted in Steam is now removed when the game is imported again. It's backed up first.
- SteamGridDB now searches by the game's current name, including names you changed in Steam or in Full Steam Ahead.
- The window can now be made smaller.

### Fixed

- Scanning froze the app on Linux.
- itch.io games weren't found when itch.io was installed as a Flatpak.
- The Flatpak version of Full Steam Ahead skipped installed Heroic games.
- The Flatpak version didn't find the settings of MiniGalaxy and Rare when they weren't installed as Flatpaks.
- Importing again replaced custom artwork in Steam with the official Steam artwork.
- Artwork chosen for a game renamed in Full Steam Ahead didn't show up in Steam.
- Tall artwork previews could overflow their box.

## [1.5.6] - 2026-10-10

### Changed

- Changed the app icon sizing.
- On Linux, the app now uses a new icon that fits in with GNOME and KDE.

## [1.5.5] - 2026-10-09

### Fixed

- Added a possible fix for the app not opening on Linux with Wayland and an NVIDIA graphics card.

## [1.5.3] - 2026-09-28

### Added

- Games can now be imported from the command line with `import`, using your saved settings and without opening a window. Use `--user` to pick a Steam user, `--source` to scan only specific sources and `--dry-run` to preview the changes without writing anything.
- Added support for right-to-left languages.

### Changed

- Cleaned up locales. If you picked a language in the settings, you may need to select it again.

## [1.5.2] - 2026-09-27

### Changed

- Made button styling consistant.

## [1.5.1] - 2026-09-27

### Added

- Custom games can now be removed from the import review.

### Changed

- Reworked the styling of ui elements.

### Fixed

- Changing settings after an import no longer leaves the app in an invalid state.
- Artwork matching and SteamGridDB searches now use a game's original launcher name, even when it has been renamed in the review.
- Fixed the styling when renaming games.

## [1.5.0] - 2026-09-27

### Added

- Games can now be renamed before importing, and reset to the name from their launcher.
- Full Steam Ahead is now available through winget.

### Changed

- Games you renamed or edited in Steam are now kept that way. Re-importing no longer reverts their name, launch options, tags or icon.
- Renamed the "Game Pass" source to "Xbox".
- Disabled the browser right-click menu inside the app.

### Fixed

- Games renamed in Steam were added a second time on the next import.
- Importing from some launchers removed the Steam collections created for the others.
- Re-importing a game reset its play time and hidden state in Steam.
- Games with the same name from different launchers were merged into one entry in the review.
- Games launched through their launcher didn't show their downloaded icon.
- The progress screen didn't show which game's artwork was being applied.
- Restoring a backup didn't restore the Proton settings changed during the import.

### Security

- Artwork is now strictly only downloaded from Steam and SteamGridDB.
- Launch arguments read from launcher data are now escaped.
- Backups and restores now strictly only touch Steam's own files.
- Proton setup no longer edits `config.vdf` when the file's layout is unexpected.

## [1.4.0] - 2026-09-17

### Added

- Heroic: Non-store games are now imported.
- Heroic: Amazon Games installs are now imported.
- The update dialog now shows the release notes for the new version.

### Changed

- Renamed the "EA app / Origin" source to "EA app". Settings carry over automatically, but its Steam collection is recreated under the new name.
- Reworked most importers to read launcher data more reliably instead of guessing at file formats.
- Failures are now logged in much more detail, so log files are far more useful for troubleshooting.

### Fixed

- itch.io: Installed games could be missed or fail to import depending on how they were set up.
- Playnite: Some games weren't detected correctly.
- Ubisoft Connect and EA app: detection is now more reliable, including through Proton on Linux.
- Bottles: Programs removed in Bottles were still imported, and their launch arguments were discarded.
- Minigalaxy: The configured install folder was ignored, and Linux-native games weren't found.
- Legendary: Games weren't found when only Rare was installed.
- Epic: Games in a Proton prefix other than the first one checked could be missed.

## [1.3.10] - 2026-09-13

### Changed

- Disabled auto updates when the app is packaged as a Flatpak.

## [1.3.9] - 2026-09-13

### Fixed

- Fixed the self updater.

## [1.3.8] - 2026-09-13

### Added

- Added a proper self updater.

### Fixed

- Fixed Game Pass detection for custom GDK application IDs (thanks @Cynary).

## [1.3.7] - 2026-09-12

### Added

- Added arm binaries.

## [1.3.6] - 2026-09-11

### Fixed

- Fixed a regression causing artwork to not apply.

## [1.3.5] - 2026-09-11

### Added

- Launch method option, to choose the whether games start directly or through their launcher by default.
- Prompt to grant Steam the permission it needs to launch non-Steam shortcuts, shown when applying if Steam is installed as a Flatpak.

### Changed

- Added proper neutral state to checkboxes.

### Fixed

- Non-Steam shortcuts failing to launch when Steam itself is installed as a Flatpak.
- Games that require Epic authentication not starting, now launching through the Epic launcher instead (thanks @Cynary).
- Deselecting one game in a source made the whole section look unselected, even with other games still selected.

## [1.3.4] - 2026-09-06

### Fixed

- Steam stopping and restarting not workin in some environments.

## [1.3.3] - 2026-09-05

### Added

- Ko-fi card on the done screen.
- Default artwork source setting (None, Steam, or SteamGridDB).

### Changed

- Flatpak is now disabled as an import source by default, since it lists every installed flatpak, not just games.
- Reorganized settings handling so defaults live only in the backend.
- Cleaned up minor UI spacing and hover-state inconsistencies.

### Fixed

- Window not draggable on Linux.
- Missing or broken source icons on Linux (Bottles, Flatpak, Heroic, Lutris, MiniGalaxy).
- Some Linux-only sources not showing up after scanning.

## [1.3.2] - 2026-09-05

### Added

- Update available modal.

## [1.3.1] - 2026-09-04

### Added

- Added a self shortcut option, useful for launching it from the Steam Deck's game mode.

### Changed

- Improved fetching of Steam artwork.

### Fixed

- Fixed font not being bundled.

## [1.3.0] - 2026-09-02

### Added

- Localization support, with community translations via Crowdin.
- Color scheme option.
- Option to delete specific artworks.
- Import, export, and reset settings.

### Changed

- Improved artwork handling when replacing or deleting existing images.
- Cleaned up and reorganized settings.

### Fixed

- Fixed window flashing blank before the webview was ready.

## [1.2.2] - 2026-08-29

### Fixed

- Version field in the debug settings tab.

## [1.2.1] - 2026-08-29

### Changed

- Changed setting tab spacing.

## [1.2.0] - 2026-08-29

### Added

- SteamGridDB browser for picking artwork, with an option to allow NSFW results.
- Sources are now configurable instead of fixed.
- Debug settings view.
- Delete all backups option.
- Single instance enforcement, so the app can't be opened twice.

### Changed

- Reworked the settings into a tabbed layout (Steam, Sources, Artwork, Backups, Debug).
- Improved backups styling.

### Fixed

- Fixed disabled sources still showing.
- Fixed review view not refreshing when settings are changed.

## [1.0.31] - 2026-08-26

### Fixed

- Fixed games launched trough proton not launching on Linux.

## [1.0.30] - 2026-08-26

### Changed

- Improved steam install location detection on linux.

### Fixed

- Fixed no steam profile found error message on linux.

[Unreleased]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.6.2...HEAD
[1.6.2]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.6.1...v1.6.2
[1.6.1]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.6.0...v1.6.1
[1.6.0]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.6...v1.6.0
[1.5.6]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.5...v1.5.6
[1.5.5]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.3...v1.5.5
[1.5.3]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.2...v1.5.3
[1.5.2]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.1...v1.5.2
[1.5.1]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.10...v1.4.0
[1.3.10]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.9...v1.3.10
[1.3.9]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.8...v1.3.9
[1.3.8]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.7...v1.3.8
[1.3.7]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.6...v1.3.7
[1.3.6]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.5...v1.3.6
[1.3.5]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.4...v1.3.5
[1.3.4]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.3...v1.3.4
[1.3.3]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.2...v1.3.3
[1.3.2]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.2.2...v1.3.0
[1.2.2]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.2.1...v1.2.2
[1.2.1]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.1.1...v1.2.0
[1.0.31]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.0.30...v1.0.31
[1.0.30]: https://github.com/creeperkatze/full-steam-ahead/compare/v1.0.29...v1.0.30
