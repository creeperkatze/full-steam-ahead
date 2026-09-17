# Full Steam Ahead

Full Steam Ahead is a Tauri desktop app (Rust backend, Vue frontend) that imports games from other launchers into Steam as non-Steam shortcuts, with artwork and collections.

This project uses pnpm. `website/` is the separate VitePress project site with its own `package.json`.

## Commands

- `pnpm dev` runs the app with auto-updates disabled.
- `pnpm test` runs the Rust tests; the frontend has no tests.
- `pnpm lint` runs ESLint, clippy and `cargo fmt --check`. `pnpm lint:fix` applies their fixes and formats.
- `pnpm typecheck` checks the frontend.
- Never run the `release:*` scripts or push tags. Releases are done by the maintainer.

## Rules

- Code behind `#[cfg(unix)]` or `#[cfg(windows)]` only compiles on that platform. Say so when a change couldn't be compiled locally; CI builds and tests on Linux, Windows and macOS.
- Prefer a maintained crate over hand-written parsing of bytes, URLs, dates or text formats.
- Launchers change: check the upstream source of open-source launchers instead of trusting existing code or memory.
- Only write a comment when the code can't explain something itself. Keep it short and simple: no em dashes, no semicolons, no nested clauses.

For how importers should find and read launcher data, see [docs/IMPORTERS.md](docs/IMPORTERS.md).
For logging, see [docs/LOGGING.md](docs/LOGGING.md).
