# Steam files

FSA writes shortcuts (`shortcuts.vdf`), artwork (the `grid` folder) and collections (Steam's cloud storage JSON). On Linux it also maps Proton in `config.vdf`. Every file is backed up before it changes (`backups::create`), with a manifest that restores it.

## Shortcut identity

- A shortcut is identified by its app id only, never by name or exe. Steam keeps the app id when a shortcut is renamed or edited, so artwork and collections stay attached.
- `shortcuts::link_existing` sets a candidate's `existing_app_id`. It looks for the linked id first, then for the id FSA would have computed from the launcher's name and the game or launcher exe.
- `steam::candidate_app_id` is the only place that decides a candidate's app id. Use it for shortcuts, artwork and collections.

## Updating a shortcut

`shortcuts::upsert` only changes what FSA owns:

- The name and start dir are always updated.
- The exe and launch options change together, and only when the exe changes.
- The icon changes only when the import writes or deletes icon artwork.
- Tags, play time, hidden state and every other field belong to the user.

A scan uses the name from Steam, so a rename there survives the next import. `original_name` keeps the launcher's name for the reset button.

## Preview and apply

Both build the updated shortcuts with `shortcuts::with_candidates`. Apply writes that list and the preview compares it with the file. Change the rules in one place so the preview can't disagree with what apply writes.

## Collections

FSA's collections use the key `user-collections.fsa-<source>`. An import only touches the collections of the sources it imports, and adds to the games already in them.

## config.vdf

Proton mappings are edited with `keyvalues-parser`. FSA only adds missing entries, and skips the file unless it has exactly one `CompatToolMapping` section.
