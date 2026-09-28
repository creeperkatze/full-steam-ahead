# Frontend and backend

The frontend calls the backend only through `src/bindings.ts`. [tauri-specta](https://github.com/specta-rs/tauri-specta) generates it from the Rust commands, events and types. Don't edit it by hand.

## Regenerating the bindings

After changing a command, an event or a type they use, run `pnpm gen:bindings` and commit the updated `src/bindings.ts`. Debug builds also export them whenever the app starts, so `pnpm dev` keeps them current too. Release builds never write the file.

## Commands

- Put the attributes in this order: `#[tauri::command]`, `#[instrument(...)]`, `#[specta::specta]`. Specta can't parse `tracing` field syntax like `%request.id`, so it has to come after `instrument`.
- Register the command in `specta_builder()` in `src-tauri/src/lib.rs`.
- Return `CommandResult<T>`. A failed command throws in the frontend.
- In the frontend, call it as `commands.someCommand(...)` from `bindings`.

## Events

- Derive `tauri_specta::Event` on the payload and register it in `specta_builder()`.
- Emit with `event.emit(&app)` and listen with `events.someEvent.listen(...)`.

## Types

- Derive `specta::Type` on every type that crosses to the frontend.
- The frontend imports types from `src/types`, which re-exports the bindings and adds frontend-only types.
- Keep these types free of serde attributes that differ by direction, like `default`, `skip_serializing` or `alias`. Specta then splits the type into `_Serialize` and `_Deserialize` variants or makes its fields optional. Handle older file formats when the file is read instead, like `Settings::from_json`.
- `u64` and `usize` are exported as `number`. Don't use them for values that can exceed 2^53.
