# Logging

Users send the session log from the logs folder when something breaks, so every backend failure must end up there with enough context to debug it. Only the Rust backend logs; the frontend doesn't write to the log.

Release builds record `info` and above; development builds also record `debug`.

## Levels

- `error`: a command failed (logged centrally when an `AppError` becomes a `CommandError`) or the app panicked.
- `warn`: something is actually broken: a file exists but can't be read or parsed, a command fails, a request errors. Include the path, command or URL, and the error.
- `info`: one summary line per user action (Steam detected, games found per source, plan applied).
- `debug`: where a launcher was found or looked for, and why a game was skipped. A launcher that isn't installed is `debug`, not `warn`.

## How

- Importers use the helpers in `importers/mod.rs` (`read_launcher_json`, `command_stdout`, …), which already apply the rules above.
- Don't swallow errors with `.ok()`, `unwrap_or_default()` or `let _ =` without logging them first, unless failure is expected and harmless.
- Skip lines that don't help debugging ("Scanning", "Checked X") and messages that repeat on every command.
- Use structured fields (`path = %path.display()`, `%error`) instead of formatting values into the message.
- On `#[instrument]`, use `skip_all` and list the few useful fields; never record whole requests, settings or API keys.
