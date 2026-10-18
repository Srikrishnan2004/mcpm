# Contributing to mcpm

Thanks for helping! Two contributions are especially easy and valuable.

## Add a server template

1. Create `catalog/servers/<id>.json` following [`catalog/schema/template.schema.json`](catalog/schema/template.schema.json).
2. Use placeholders:
   - `{{setting}}`: a setting value
   - `{{secret:key}}`: a secret from the keychain. **Only allowed in `env` and `headers`**, never in `args` or `url`, because those are visible in process lists.
   - `{{#flag}}…{{/flag}}`: included only when a boolean setting is true.
3. Register the file in `BUILTIN` in `crates/core/src/catalog.rs`.
4. Run `cargo test -p mcpm-core`.

## Add a client adapter

1. Most clients are a JSON file with a map of servers: add a `JsonClient` entry in `crates/core/src/adapters.rs` (`all()`).
2. Other formats (TOML, YAML): implement the `ClientAdapter` trait in a new module.
3. Add fixture tests proving that user entries and unrelated keys survive a write.
4. Link the official docs for the config path and format in your PR.

## Ground rules

- Never write secret values into client config files.
- Never modify config entries mcpm didn't create (the manifest tracks ownership).
- The shim (`mcpm run`) must not print to stdout; stdout belongs to the MCP protocol.
- `cargo fmt`, `cargo clippy --workspace`, `cargo test --workspace`, and `pnpm typecheck` must pass.

## UI

Components live in `apps/desktop/src/components/ui` and follow the token system in `src/styles/tokens.css` (see DESIGN.md §9). Don't copy Vercel component code or icons; use `lucide-react` for icons.
