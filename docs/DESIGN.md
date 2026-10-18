# mcpm — Design Document

> **mcpm** — one desktop app to manage MCP servers across every AI IDE and CLI.
> Status: draft v0.1 · Last updated: 2026-10-18 · License: MIT OR Apache-2.0

---

## 1. Problem

Every AI coding client (Claude Code, Cursor, VS Code Copilot, Windsurf/Devin Desktop, Antigravity, Codex, Gemini CLI, Claude Desktop, …) stores MCP server configuration in its **own file, path, and format**. Adding one server (e.g. Supabase) means editing N files with N slightly different schemas, copying secrets into each, and keeping them in sync by hand.

A second, sharper problem: **one server type, many accounts.** A developer with two Supabase projects (client work + personal) has two different access tokens / project refs. Today the typical workflow is: delete the Supabase entry, paste the other one, restart the IDE — every time they switch projects.

## 2. Goals / Non-goals

### Goals

1. **Single source of truth** for MCP servers on a machine.
2. **Named instances** of the same server ("namespaces"): `supabase@clientA`, `supabase@personal`.
3. **Per-project remap**: bind a project folder to a specific instance and switch any time with one click.
4. **Sync to all clients** in each client's native format, at global and project scope.
5. **No secrets in config files.** Credentials live in the OS keychain only.
6. **Safe writes**: never clobber user content; back up before every write; detect drift.
7. **Open source & extensible**: community can add server templates and client adapters via small PRs.
8. Cross-platform: Windows, macOS, Linux.

### Non-goals (v1)

- Hosting/running MCP servers remotely.
- A public server marketplace with accounts (catalog is a folder in the repo).
- Team/cloud sync of configs (possible later via git-friendly export).
- Editing client settings unrelated to MCP.

## 3. Core concepts

| Concept | Definition | Example |
|---|---|---|
| **Template** | A server *type*: how to launch it, which settings/secrets it needs. Lives in the catalog or is user-defined. | `supabase` |
| **Instance** | A named, configured copy of a template, with its own settings and secrets. Identified as `<template>@<name>`. | `supabase@clientA` |
| **Client** | An AI IDE/CLI that consumes MCP config. Handled by an **adapter**. | Cursor |
| **Scope** | Where a client entry is written: `global` (user-level file) or `project` (file inside a repo). | `.cursor/mcp.json` |
| **Project** | A folder registered in mcpm. | `D:\work\shop` |
| **Binding** | Mapping inside a project: *entry name* → *instance*, plus which clients to write to. | `supabase` → `supabase@clientA` in Cursor + Claude Code |
| **Profile** (global) | The set of instances enabled at global scope per client. | Claude Desktop: `github@me`, `supabase@personal` |

**Remap** = change a binding's instance → mcpm rewrites the affected client files.

The *entry name* the IDE sees stays stable (`supabase`) regardless of which instance is behind it, so tool names (`mcp__supabase__execute_sql`, etc.) and the user's prompts don't change between projects.

## 4. Architecture

```
┌──────────────────────────────── mcpm desktop app (Tauri v2) ───────────────────────────────┐
│                                                                                             │
│  ┌────────────── UI (React + TS + Tailwind, Geist design language) ──────────────┐          │
│  │ Servers · Instances · Projects · Clients · Catalog · Activity · Settings      │          │
│  └─────────────────────────────────┬─────────────────────────────────────────────┘          │
│                                    │ Tauri commands (typed, via tauri-specta)                │
│  ┌─────────────────────────────────▼─────────────────────────────────────────────┐          │
│  │                         mcpm-core (Rust library)                              │          │
│  │  Registry   Bindings   Catalog   SecretStore   SyncEngine   Backup   Doctor   │          │
│  │                              │                                                │          │
│  │                    ┌─────────▼──────────┐                                     │          │
│  │                    │  ClientAdapter     │  claude_code · claude_desktop ·     │          │
│  │                    │  (trait, 1 file    │  cursor · vscode · windsurf ·       │          │
│  │                    │   per client)      │  antigravity · codex · gemini · zed │          │
│  │                    └─────────┬──────────┘                                     │          │
│  └──────────────────────────────┼────────────────────────────────────────────────┘          │
└─────────────────────────────────┼───────────────────────────────────────────────────────────┘
                                  │ writes (merge, backup, manifest)
                                  ▼
   ~/.claude.json  .mcp.json  ~/.cursor/mcp.json  .vscode/mcp.json  ~/.codex/config.toml  …
                                  │
                                  │ each managed entry is:  command = "mcpm", args = ["run", "supabase@clientA"]
                                  ▼
┌─────────────────────────── IDE spawns ───────────────────────────┐
│  mcpm (CLI / shim, separate small binary, same mcpm-core)         │
│   1. resolve instance from registry                               │
│   2. fetch secrets from OS keychain                               │
│   3a. stdio server: spawn real command with env, pipe stdio       │
│   3b. remote server: act as stdio ↔ Streamable HTTP bridge        │
└───────────────────────────────────────────────────────────────────┘
```

### 4.1 Components

| Component | Crate / folder | Responsibility |
|---|---|---|
| `mcpm-core` | `crates/core` | Domain model, registry & bindings persistence, catalog loading, secret store, client adapters, sync engine, backups, doctor checks. No UI, no Tauri dependency. |
| `mcpm` CLI | `crates/cli` | The launcher shim (`mcpm run`) plus a scriptable CLI (`mcpm list`, `mcpm bind`, `mcpm sync`, `mcpm doctor`). Small, fast-starting binary. |
| `mcpm-app` | `apps/desktop/src-tauri` | Tauri v2 shell: exposes core as commands, file watcher, tray, updater. Bundles `mcpm` CLI as a sidecar. |
| UI | `apps/desktop/src` | React + TypeScript + Vite + Tailwind v4, Geist-style component kit. |
| Catalog | `catalog/` | Community-maintained server templates (`*.json`), validated by JSON Schema in CI. |

### 4.2 Why a launcher shim

Config files are frequently committed to git (`.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json`, `.codex/config.toml`). Writing tokens into them leaks secrets. Clients also disagree on env-var interpolation (`${VAR}` vs `${env:VAR}` vs none — see §7), so there's no portable "reference a secret" syntax.

The shim solves both:

- Every managed entry is identical across clients: `command: "mcpm"`, `args: ["run", "<instance>"]`.
- Secrets are injected **at spawn time** from the keychain; nothing sensitive touches disk.
- Remapping = changing one string in `args`.
- Remote servers work even in clients that lack native HTTP config (Claude Desktop) or use odd field names (`serverUrl`, `httpUrl`).

Trade-off: one extra process per server and a hard dependency on `mcpm` being installed. Startup overhead target: < 30 ms before the child is spawned.

**Escape hatch:** per instance, users can choose `materialize` mode, which writes the real command/url into the client config (using the client's interpolation syntax where possible, never raw secrets into project scope). Useful for clients/environments where the shim can't run.

### 4.3 Shim path resolution

| Scope | `command` written | Reason |
|---|---|---|
| Global configs | Absolute path to installed `mcpm` binary | GUI apps (Claude Desktop, macOS apps launched from Dock) often don't inherit shell `PATH`. |
| Project configs | Bare `mcpm` | Committed files must be portable across machines/OSes. The installer adds `mcpm` to `PATH`. |

`mcpm doctor` detects stale absolute paths (after reinstall/move) and offers "repair all".

### 4.4 Shim behaviour (`mcpm run <instance>`)

1. Load `registry.json` (read-only, no lock needed). Unknown instance → write a clear error to stderr and exit 2 (clients surface stderr in their MCP logs).
2. Resolve template + instance settings → final `command`, `args`, `env`, `cwd`, or `url` + `headers`.
3. Fetch each secret from the keychain (`service = "mcpm"`, `account = "<instance>/<KEY>"`).
4. **stdio transport:** spawn child with merged env (inherit parent env + instance env + secrets), pipe stdin/stdout/stderr byte-for-byte, forward signals, exit with child's code. On Windows, resolve `npx`/`uvx` via `PATHEXT` (spawn `npx.cmd`), and use a Job Object so the child dies with the shim.
5. **remote transport:** run a stdio ↔ Streamable HTTP bridge (JSON-RPC lines on stdin → POST to server; SSE/JSON responses → stdout). Handles `Mcp-Session-Id`, auth headers, and OAuth (token cache in keychain, browser-based authorization via `mcpm auth <instance>`).
6. Optional `--log` writes a redacted transcript to `<data>/logs/<instance>.log` for the Activity view.

### 4.5 Future: "slot mode" (v2)

In pinned mode (v1) the instance name appears in the project file. That's fine for solo users but leaks a private label into shared repos. Slot mode writes `args: ["run", "--slot", "supabase", "--project", "${workspaceFolder}"]` (using each client's variable syntax, falling back to `cwd`), and the shim resolves the slot through the local binding. Remapping then requires **no file writes and no IDE restart**. Deferred because project-directory discovery differs per client.

## 5. Data model & storage

All non-secret state lives in the OS app-data dir (`dirs::config_dir()/mcpm/`):

- Windows: `%APPDATA%\mcpm\`
- macOS: `~/Library/Application Support/mcpm/`
- Linux: `$XDG_CONFIG_HOME/mcpm/` (default `~/.config/mcpm/`)

```
mcpm/
├── registry.json     # instances (no secrets)
├── projects.json     # projects + bindings
├── profiles.json     # global-scope enablement per client
├── manifest.json     # what mcpm wrote where (for safe merge & drift detection)
├── templates/        # user-defined templates (same schema as catalog)
├── backups/          # timestamped copies of every client file before write
└── logs/
```

Every file has `"version": 1` for migrations. Writes are atomic (write temp → fsync → rename) and guarded by a lock file shared by app and CLI.

### 5.1 Template (catalog entry)

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/<org>/mcpm/main/catalog/schema/template.schema.json",
  "id": "supabase",
  "name": "Supabase",
  "description": "Query and manage Supabase projects.",
  "homepage": "https://supabase.com/docs/guides/getting-started/mcp",
  "icon": "supabase.svg",
  "tags": ["database", "postgres"],
  "transports": {
    "stdio": {
      "command": "npx",
      "args": ["-y", "@supabase/mcp-server-supabase@latest", "--project-ref={{project_ref}}", "{{#read_only}}--read-only{{/read_only}}"],
      "env": { "SUPABASE_ACCESS_TOKEN": "{{secret:access_token}}" }
    },
    "http": {
      "url": "https://mcp.supabase.com/mcp?project_ref={{project_ref}}{{#read_only}}&read_only=true{{/read_only}}",
      "auth": "oauth"
    }
  },
  "defaultTransport": "stdio",
  "settings": [
    { "key": "project_ref", "label": "Project ref", "type": "string", "required": true },
    { "key": "read_only", "label": "Read-only", "type": "boolean", "default": true }
  ],
  "secrets": [
    { "key": "access_token", "label": "Personal access token", "required": true,
      "help": "Supabase dashboard → Account → Access Tokens" }
  ]
}
```

Placeholders: `{{setting}}`, `{{secret:key}}` (only legal inside `env`/`headers`, never in `args`/`url`, to keep secrets off process listings), `{{#flag}}…{{/flag}}` for booleans. Empty args after rendering are dropped.

### 5.2 Instance (`registry.json`)

```json
{
  "version": 1,
  "instances": [
    {
      "id": "supabase@clientA",
      "template": "supabase",
      "label": "Client A – production",
      "transport": "stdio",
      "settings": { "project_ref": "abcd1234", "read_only": true },
      "secrets": ["access_token"],
      "mode": "shim",
      "createdAt": "2026-10-18T10:00:00Z"
    }
  ]
}
```

`secrets` lists *which keys exist* in the keychain — never values. Instance IDs match `^[a-z0-9][a-z0-9-]*@[a-z0-9][a-z0-9-]*$` (no underscores; Gemini builds tool names as `mcp_<server>_<tool>`).

### 5.3 Project & bindings (`projects.json`)

```json
{
  "version": 1,
  "projects": [
    {
      "id": "p_01J…",
      "path": "D:\\work\\shop",
      "name": "shop",
      "bindings": [
        { "entry": "supabase", "instance": "supabase@clientA",
          "clients": ["claude_code", "cursor", "vscode"] }
      ]
    }
  ]
}
```

### 5.4 Global profiles (`profiles.json`)

```json
{
  "version": 1,
  "clients": {
    "claude_desktop": [{ "entry": "supabase", "instance": "supabase@personal" }],
    "claude_code":    [{ "entry": "github",   "instance": "github@me" }]
  }
}
```

### 5.5 Manifest (`manifest.json`)

```json
{
  "version": 1,
  "files": {
    "D:\\work\\shop\\.cursor\\mcp.json": {
      "client": "cursor",
      "entries": { "supabase": { "hash": "sha256:…", "instance": "supabase@clientA" } }
    }
  }
}
```

The manifest is how mcpm knows which entries it **owns**. Entries not in the manifest are user-owned and never modified. If an owned entry's on-disk hash differs from the recorded hash, the user edited it → **drift**, surfaced in UI with "keep theirs / overwrite / adopt as new instance".

### 5.6 Secrets

- Crate: [`keyring`](https://crates.io/crates/keyring) (Windows Credential Manager, macOS Keychain, Linux Secret Service / kwallet).
- Key: `service = "mcpm"`, `account = "<instance-id>/<secret-key>"`.
- UI never displays stored values after save (only "set · updated 3d ago" + replace/clear). Reveal requires an explicit click, re-reading from keychain.
- Instance delete → delete its keychain items.
- Linux without a Secret Service: mcpm refuses to store secrets in plain text and shows guidance (install gnome-keyring/kwallet); an opt-in encrypted-file fallback (age, passphrase) may come later.
- OAuth tokens for remote instances are stored the same way (`account = "<instance>/oauth"`).

## 6. Client adapters

```rust
pub trait ClientAdapter: Send + Sync {
    fn id(&self) -> ClientId;                         // "cursor"
    fn display_name(&self) -> &'static str;           // "Cursor"
    fn detect(&self) -> Detection;                    // installed? version? config paths found?
    fn config_paths(&self, scope: &Scope) -> Vec<PathBuf>; // ordered; first existing wins for read
    fn supports(&self, scope: ScopeKind) -> bool;
    fn capabilities(&self) -> Capabilities;           // remote_native, interpolation, disable_flag, hot_reload
    fn read(&self, path: &Path) -> Result<Vec<ClientEntry>>;           // parse existing entries
    fn render(&self, entry: &ManagedEntry) -> Result<RenderedEntry>;   // our model → client shape
    fn write(&self, path: &Path, plan: &WritePlan) -> Result<WriteReport>; // format-preserving merge
    fn reload_hint(&self) -> ReloadHint;              // "restart app", "run /mcp", "auto"
}
```

- JSON files: parse with a format-preserving JSONC editor (`jsonc-parser` crate with CST editing) so comments, key order, and indentation survive.
- TOML (Codex): `toml_edit`.
- Each adapter lives in `crates/core/src/adapters/<client>.rs` with fixture-based tests (`tests/fixtures/<client>/*.input.json` → `*.expected.json`).

### 6.1 Write algorithm (all adapters)

1. Compute desired state for the file: owned entries from bindings/profiles.
2. Read current file (missing → create with minimal skeleton).
3. Diff against manifest: add / update / remove owned entries; detect drift; detect name collisions with user-owned entries (→ error, ask user to rename the binding's entry name or adopt the existing entry).
4. Show plan in UI (diff preview) unless "auto-apply" is on.
5. Back up the file to `backups/<timestamp>/<client>/<sanitized-path>`.
6. Apply via format-preserving editor; atomic write.
7. Update manifest; emit Activity event with reload hint.

## 7. Supported clients (v1)

Verified against official docs on 2026-10-18. ⚠ = unverified/conflicting, adapter probes multiple paths.

| Client | Global path | Project path | Format / key | Remote field | Interpolation | Disable flag | Reload |
|---|---|---|---|---|---|---|---|
| **Claude Code** | `~/.claude.json` → `mcpServers` (local scope: `projects["<abs>"].mcpServers`) | `.mcp.json` | JSON / `mcpServers`, `type: stdio\|http\|sse` | `url` + `type: "http"` | `${VAR}`, `${VAR:-d}` | none in file (`disabledMcpServers` state) | new session / `/mcp` |
| **Claude Desktop** | `%APPDATA%\Claude\claude_desktop_config.json` · `~/Library/Application Support/Claude/…` · ⚠ `~/.config/Claude/…` | — | JSON / `mcpServers` | not in file (shim bridges) | none | none | full restart |
| **Cursor** | `~/.cursor/mcp.json` | `.cursor/mcp.json` | JSON / `mcpServers` | `url` | `${env:X}`, `${workspaceFolder}` | none (UI) | restart |
| **VS Code (Copilot)** | user profile `mcp.json` (`%APPDATA%\Code\User\mcp.json` ⚠) | `.vscode/mcp.json`; portable `.mcp.json` | JSONC / `servers` (+`inputs`); portable uses `mcpServers` | `url` + `type: "http"` | `${env:X}`, `${input:id}`, `${workspaceFolder}` | none (UI) | auto (`chat.mcp.autostart`) |
| **Windsurf / Devin Desktop** ⚠ | probe: `%APPDATA%\devin\mcp_config.json` / `~/.config/devin/…`, `~/.codeium/mcp_config.json`, legacy `~/.codeium/windsurf/mcp_config.json` | — | JSON / `mcpServers` | `serverUrl` (or `url`) | `${env:X}`, `${file:p}` | `disabledTools` only | ⚠ |
| **Antigravity** | `~/.gemini/config/mcp_config.json` (⚠ legacy `~/.gemini/antigravity/mcp_config.json`) | `.agents/mcp_config.json` | JSON / `mcpServers` | `serverUrl` only | none | `"disabled": true` | `/mcp`, refresh |
| **Codex** | `~/.codex/config.toml` (`CODEX_HOME`) | `.codex/config.toml` (trusted projects) | TOML / `[mcp_servers.<name>]` | `url` (no `type`) | none (env names via `env_vars`, `bearer_token_env_var`) | `enabled = false` | restart |
| **Gemini CLI** | `~/.gemini/settings.json` | `.gemini/settings.json` | JSON / `mcpServers` | `httpUrl` (HTTP) / `url` (SSE) | `$VAR`, `${VAR}`, `${VAR:-d}` | `mcp-server-enablement.json` | `/mcp refresh` ⚠ |
| **Zed** (v1.1) | ⚠ `~/.config/zed/settings.json` | ⚠ `.zed/settings.json` | JSONC / `context_servers` | `url` | ⚠ | none | ⚠ |

Notes:

- Because managed entries use the shim, adapters mostly need only the stdio shape (`command` + `args`). The remote-field/interpolation columns matter for `materialize` mode and for **import** of existing configs.
- Market changes in 2026: Windsurf was rebranded Devin Desktop; Gemini CLI was superseded by Antigravity CLI for free tiers but remains supported. Adapter detection must probe old and new paths and report which one is active.
- Cline, Continue, JetBrains AI Assistant: community adapters welcome (v1.x).

## 8. Key flows

### 8.1 First run / import

1. Detect installed clients (adapter `detect()`).
2. Parse every global + known project config, collect entries.
3. Group duplicates by (command, args, url) signature; match against catalog templates.
4. Show an import wizard: "We found Supabase in 4 clients with 2 distinct tokens → create `supabase@a` and `supabase@b`?"
5. Secrets found in plaintext env/headers are moved into the keychain; the source entries are replaced with shim entries (with backup, after confirmation).

### 8.2 Add instance

Catalog → pick template → fill settings & secrets (form generated from template schema) → **Test connection** (shim spawns server, runs `initialize` + `tools/list`, shows tool count & latency) → save.

### 8.3 Bind / remap in a project

Projects → open folder → "Add server" picks an entry name + instance + target clients → preview diff → apply.
Remap: change dropdown `supabase: [clientA ▾]` → preview → apply → toast with per-client reload hints ("Cursor: restart required", "VS Code: reloads automatically").

### 8.4 Global profiles

Clients view → per client, toggle instances on/off at global scope. Same write algorithm.

### 8.5 Doctor

Checks: shim path valid; `node`/`npx`/`uvx`/`docker` available for templates that need them; keychain reachable; drift; orphaned keychain items; configs that still contain plaintext secrets; duplicate entry names across global + project scope (client precedence explained).

## 9. UI

mcpm adopts the **visual language of Vercel's Geist design system** (monochrome surfaces, hairline borders, Geist type, restrained accent color). It does **not** use Vercel's name, logo, icons, or component code — tokens are renamed and all components are written from scratch. Values below were extracted from the CSS served at vercel.com/geist on 2026-10-18.

### 9.1 Principles

1. **Monochrome first.** Gray scale carries 95% of the UI; color only for status (green = synced, amber = drift/restart needed, red = error, blue = focus/info).
2. **Hairlines, not boxes.** Borders are 1px `box-shadow` rings using alpha grays, so they work on any surface.
3. **Dense but calm.** 36px controls, 14px body text, 4px spacing grid.
4. **Light + dark** from day one (follow OS, overridable). Theme via `.dark` class on `<html>`.
5. **Keyboard-first.** ⌘/Ctrl-K command palette (remap, add instance, open project), visible focus ring everywhere.

### 9.2 Tokens (CSS variables, `apps/desktop/src/styles/tokens.css`)

Step semantics (all color scales): 100 bg · 200 hover bg · 300 active bg · 400 border · 500 hover border · 600 active border · 700 solid bg · 800 solid hover · 900 secondary text/icon · 1000 primary text/icon.

| Token | Light | Dark |
|---|---|---|
| `--bg-100` (surface) | `#FFFFFF` | `#0A0A0A` |
| `--bg-200` (page / sidebar) | `#FAFAFA` | `#000000` |
| `--gray-100 … 300` | `#F2F2F2 #EBEBEB #E6E6E6` | `#1A1A1A #1F1F1F #292929` |
| `--gray-400 … 600` | `#EBEBEB #C9C9C9 #A8A8A8` | `#2E2E2E #454545 #878787` |
| `--gray-700 / 800` | `#8F8F8F #7D7D7D` | `#8F8F8F #7D7D7D` |
| `--gray-900 / 1000` | `#4D4D4D #171717` | `#A1A1A1 #EDEDED` |
| `--gray-a-100 … 1000` | black @ `0D 14 1A 14 36 57 70 82 B3 E8` | white @ `0F 17 21 24 3D 82 8A 78 9C EB` |
| `--blue-100/200/700/800/900/1000` | `#F0F7FF #EBF5FF #0072F5 #0062D1 #0068D6 #00254C` | `#0F1C2E #10233D #0072F5 #0062D1 #52A8FF #EBF6FF` |
| `--red-…` | `#FFF0F0 #FFEBEB #E5484D #DA2F35 #CB2A2F #391417` | `#2A1314 #3C1618 #E5484D #D93036 #FF6166 #FEECEE` |
| `--amber-…` | `#FFF6E5 #FFF4D6 #FFB224 #FF990A #A35200 #4E2009` | `#291800 #331B00 #FFB224 #FF990A #F2A20D #FEF3DC` |
| `--green-…` | `#EFFBEF #EBFAEB #45A557 #398E4A #297A3A #1B311E` | `#0B2212 #0F2E18 #45A557 #398E4A #62C073 #E5FBEA` |

Other tokens:

- `--focus-ring: 0 0 0 2px var(--bg-100), 0 0 0 4px var(--focus)` with `--focus` = blue-700 (light) / blue-900 (dark).
- `--ring-border: 0 0 0 1px var(--gray-a-400)`; input focus: `0 0 0 1px var(--gray-a-600), 0 0 0 4px` black/16% (white/24% dark).
- Radii: `6px` controls · `8px` large buttons / cards · `12px` menus, modals, panels · `9999px` badges.
- Shadows: `small`, `medium`, `large`, `menu`, `modal` as Geist materials (hairline + layered soft shadow).
- Motion: `150ms ease` for hover/press; `200ms cubic-bezier(.2,.8,.2,1)` for menus/modals; respect `prefers-reduced-motion`.

Tailwind v4 `@theme` maps these to utilities (`bg-bg-100`, `text-gray-900`, `shadow-menu`, …).

### 9.3 Typography

- **Geist** (sans) and **Geist Mono** via `@fontsource-variable/geist` and `@fontsource-variable/geist-mono` (OFL-1.1, bundled locally — works offline in Tauri). Include `OFL.txt` in third-party notices.
- Mono for: instance IDs (`supabase@clientA`), paths, commands, JSON previews.

| Class | Size / line-height | Weight | Tracking |
|---|---|---|---|
| `heading-24` | 24/32 | 600 | -0.96px |
| `heading-20` | 20/26 | 600 | -0.4px |
| `heading-16` | 16/24 | 600 | -0.32px |
| `heading-14` | 14/20 | 600 | -0.28px |
| `label-14` / `label-13` / `label-12` | 14/20 · 13/16 · 12/16 | 400 (500 strong) | 0 |
| `copy-14` / `copy-13` | 14/20 · 13/18 | 400 | 0 |
| `button-14` / `button-12` | 14/20 · 12/16 | 500 | 0 |
| `label-13-mono` | 13/20 | 400 | 0 |

### 9.4 Component kit (`apps/desktop/src/components/ui/`)

Written from scratch on top of Radix primitives (accessible behavior) + Tailwind. Icons: **lucide-react** (ISC), 16px, `strokeWidth={1.5}` — Geist icons are not openly licensed.

| Component | Spec |
|---|---|
| `Button` | Heights sm 32 / md 36 / lg 40; px 6/10/14; radius 6 (lg 8). Variants: **primary** gray-1000 bg / bg-100 text; **secondary** bg-100 + ring-border, hover gray-a-200; **tertiary** transparent, hover gray-a-200; **error** red-800, hover red-900; **warning** amber-800, dark text. Disabled: gray-100 bg, gray-700 text. Props: `prefix`, `suffix`, `loading`, `iconOnly` (+ `aria-label`). |
| `Input` | 36px, px 12, 14px, radius 6, ring-border → hover gray-a-500 → focus input-focus ring. Placeholder gray-700. Prefix/suffix addons. `Secret` variant: masked, reveal button, "stored in keychain" hint. |
| `Select` / `Combobox` | Same ring as input; chevron right; used for instance pickers (shows mono instance ID + label). |
| `Toggle` | 40×24 track, radius 14, off gray-400, on gray-1000 (green-700 in client enable lists). |
| `SegmentedControl` | Scope switcher (Global / Project), items 14px/500, selected gray-100 bg radius 4. |
| `Badge` | Pill, sm 20px / md 24px; subtle (color-200 bg, color-900 text) for statuses: `synced` green, `drift` amber, `restart needed` amber, `error` red, `unverified` gray. |
| `Tabs` | Underline: 14px gray-900, selected gray-1000 + 2px bottom border. |
| `Menu` | 12px radius, menu shadow, 6px padding, rows 36px / px 8 / radius 6, hover gray-a-100. |
| `Modal` / `Sheet` | bg-100, radius 12, modal shadow, padding 24, backdrop bg-200 @ 80%. Footer strip bg-200 with top hairline, right-aligned buttons. |
| `Toast` | Bottom-right stack, ~356px, menu material; carries reload hints with per-client actions. |
| `Table` | 14px gray-900; header 36px, weight 500; rows hairline gray-400, hover gray-a-100; last column right-aligned. |
| `EmptyState` | Centered card, icon tile (ring gray-a-400, radius 8), heading-16 + copy-14 (max 340px), one action. |
| `DiffView` | Mono 13px, added lines green-100 bg / green-900 text, removed red-100 / red-900. Used before every write. |
| `CommandPalette` | Modal material, 560px wide, input 48px, results rows 40px. |

### 9.5 Layout

```
┌──────────────┬─────────────────────────────────────────────────────────────┐
│ ◆ mcpm       │  Projects / shop                         [⌘K]  [+ Add]      │  ← 56px header, hairline bottom
│              ├─────────────────────────────────────────────────────────────┤
│ Overview     │  shop                                    ● 3 synced  ▲ 1 drift│
│ Servers      │  D:\work\shop                                               │
│ Instances    │  ─ Bindings ─ Clients ─ History ─                           │
│ Projects  ●  │  ┌─────────────────────────────────────────────────────────┐│
│ Clients      │  │ ENTRY      INSTANCE                    CLIENTS     STATUS ││
│ Catalog      │  │ supabase   [supabase@clientA      ▾]   ◧ ◩ ◨      synced ││
│ Activity     │  │ github     [github@work           ▾]   ◧ ◩        drift  ││
│              │  │ postgres   [postgres@local        ▾]   ◧          synced ││
│              │  └─────────────────────────────────────────────────────────┘│
│ ──────────── │                                                             │
│ Settings     │                                    [Preview changes] [Apply] │
│ ◐ Theme      │                                                             │
└──────────────┴─────────────────────────────────────────────────────────────┘
  240px, bg-200,     content max-width 1200, padding 24, surface bg-100
  hairline right
```

- Sidebar: 240px, bg-200, right hairline; items 32px, px 10, radius 6, 14px gray-900, 16px icons; hover gray-a-100; active gray-a-200 + gray-1000 + weight 500; section labels `label-12` gray-700.
- Windows: custom title bar (Tauri `decorations: false` + drag region) matching header height; macOS: overlay traffic lights.
- Min window 960×640; content collapses sidebar to icons below 1100px.

### 9.6 Screens

| Screen | Purpose |
|---|---|
| **Overview** | Detected clients (logos as generic icons + names), counts of instances/projects, items needing attention (drift, restart needed, plaintext secrets found). |
| **Servers** | Templates in use (installed from catalog or custom), grouped instance counts. |
| **Instances** | Table of all instances; create/edit drawer generated from template schema; "Test connection" shows tools list + latency. |
| **Projects** | List of registered folders → detail page with bindings table (remap dropdown inline), client checkboxes, diff preview, apply. |
| **Clients** | Per-client card: detected path(s), scope support, reload behaviour, global profile toggles. |
| **Catalog** | Searchable grid of templates (name, description, tags, "verified" badge). |
| **Activity** | Timeline of writes, backups (restore button), shim launches & errors. |
| **Settings** | Theme, shim path, auto-apply, backup retention, telemetry (off by default, none in v1). |

### 9.7 Brand

Own wordmark "mcpm" set in Geist Mono 600, with a simple glyph (◆-like node/plug mark, TBD). No Vercel triangle, wordmark, or product names anywhere in the app.

## 10. Tauri app specifics

- **Tauri v2**, capabilities locked down: no shell plugin exposed to the webview; fs access only through core commands.
- Plugins: `tauri-plugin-dialog` (folder picker), `tauri-plugin-updater`, `tauri-plugin-single-instance`, `tauri-plugin-window-state`, tray icon (quick remap from the tray menu).
- Typed IPC via `tauri-specta` → generated `bindings.ts`.
- `mcpm` CLI bundled through `bundle.externalBin`; installer adds it to `PATH` (Windows: NSIS hook; macOS: offer symlink to `/usr/local/bin`; Linux: deb/rpm put it in `/usr/bin`).
- File watcher (`notify` crate) on all managed files → live drift detection.

## 11. Security

- Secrets only in OS keychain; never logged; redacted in Activity transcripts (header/env values matching stored secrets are masked).
- Secrets never placed in `args` or `url` (visible in process lists).
- Webview cannot read secrets back except via an explicit "reveal" command per secret.
- Catalog templates are data only (no scripts). Templates from outside the repo show an "unverified" badge and a full command preview before first run.
- Writes outside known client paths are rejected by core.
- Signed releases (Windows Authenticode when available, macOS notarization), SHA-256 checksums published.

## 12. Repository layout

```
mcpm/
├── Cargo.toml                 # workspace
├── crates/
│   ├── core/                  # mcpm-core library
│   │   └── src/{model,registry,catalog,secrets,sync,backup,doctor}.rs, adapters/*.rs
│   └── cli/                   # `mcpm` binary (shim + CLI)
├── apps/desktop/
│   ├── src-tauri/             # Tauri app crate
│   └── src/                   # React UI
├── catalog/
│   ├── schema/template.schema.json
│   └── servers/*.json
├── docs/
├── .github/workflows/         # ci.yml (fmt, clippy, test, catalog lint), release.yml
├── CONTRIBUTING.md
├── LICENSE-MIT · LICENSE-APACHE
└── README.md
```

## 13. Testing

- **Core:** unit tests per adapter with golden fixtures (round-trip must preserve comments and unknown keys); property tests for merge idempotency (`apply(apply(x)) == apply(x)`).
- **Shim:** integration test with a tiny echo MCP server (stdio) and a mock Streamable HTTP server.
- **Keychain:** trait `SecretStore` with an in-memory impl for tests.
- **UI:** Vitest + Testing Library for components; Playwright smoke test against `tauri dev` (later).
- **CI matrix:** windows-latest, macos-latest, ubuntu-latest.

## 14. Roadmap

| Milestone | Scope |
|---|---|
| **v0.1** | Core model, registry, keychain, shim (stdio), adapters: Claude Code, Cursor, VS Code, Claude Desktop. Projects + remap. Basic UI. |
| **v0.2** | Codex (TOML), Gemini CLI, Antigravity, Windsurf/Devin adapters. Import wizard. Drift detection. Backups UI. |
| **v0.3** | Remote transport bridge + OAuth. Test connection. Activity log. Tray quick-remap. |
| **v0.4** | Slot mode (no-write remap). Zed adapter. `materialize` mode. |
| **v1.0** | Signed installers, auto-update, docs site, catalog of 30+ servers. |

## 15. Implementation status (scaffold, 2026-10-18)

| Area | State |
|---|---|
| `mcpm-core` model, registry, catalog, keychain, rendering, project sync, backups, manifest | Done, 11 unit tests |
| JSON adapters: Claude Code, Claude Desktop, Cursor, VS Code, Windsurf/Devin, Antigravity, Gemini CLI | Done (plain JSON; JSONC comment preservation pending) |
| Codex (TOML) adapter | Pending (v0.2) |
| Shim `mcpm run` (stdio) | Done, smoke-tested with filesystem server |
| Shim remote bridge + OAuth | Pending (v0.3) |
| Global profiles (global-scope writes with absolute shim path) | Pending |
| Drift UI, diff preview, import wizard, doctor | Pending |
| Desktop UI: Overview, Projects (bind/remap/unbind), Instances, Clients, Catalog | Done |
| Typed IPC via `tauri-specta` | Not yet — hand-written `src/lib/api.ts` mirrors Rust types |
| Bundling `mcpm` CLI as Tauri sidecar + PATH install | Pending; dev uses `cargo install --path crates/cli` |
| Custom title bar, command palette, tray | Pending |

## 16. Open questions

1. Should project bindings be shareable (a committed `.mcpm.json` declaring *required entries* without instance names) so teammates get prompted to bind their own instances? (Leaning yes, pairs with slot mode.)
2. VS Code: write `.vscode/mcp.json` (`servers`) or the newer portable `.mcp.json`? Default `.vscode/mcp.json`, option to switch — note Claude Code also reads `.mcp.json`, so the portable file serves both.
3. Remote servers with native client OAuth (e.g. hosted Supabase in Cursor): should mcpm prefer the client's own OAuth (materialize URL, no shim) to avoid a second login? Per-instance toggle likely.
