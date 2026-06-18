# Architecture

SaveMeGB is a Tauri 2 app. The UI is HTML/CSS/JS, the engine is Rust. They communicate via Tauri IPC.

## High level

```
┌──────────────────────────────────────────────────────────────┐
│  Tauri main process (Rust)                                   │
│                                                              │
│  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌────────┐ ┌──────────┐  │
│  │registry │ │ manifest │ │scanner │ │ safe_  │ │ settings │  │
│  │  .rs    │ │   .rs    │ │  .rs   │ │delete.rs│ │   .rs    │  │
│  └────┬────┘ └────┬─────┘ └────┬───┘ └────┬───┘ └────┬─────┘  │
│       │           │            │          │           │        │
│       └───────────┴─────┬──────┴──────────┘           │        │
│                         │                              │        │
│                  ┌──────▼──────┐                ┌──────▼──────┐ │
│                  │   lib.rs    │◄───────────────│   model.rs  │ │
│                  │ (Tauri cmd) │                │  (types)    │ │
│                  └──────┬──────┘                └─────────────┘ │
│                         │                                      │
│                  ┌──────▼──────┐                               │
│                  │   progress │                               │
│                  │    .rs      │                               │
│                  └─────────────┘                               │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri IPC (invoke / events)
┌──────────────────────────▼───────────────────────────────────┐
│  WebView (HTML/CSS/JS)                                       │
│                                                              │
│  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌────────┐ ┌──────────┐  │
│  │ main.js │ │index.html│ │styles. │ │Settings│ │ History  │  │
│  │         │ │          │ │ css    │ │  UI    │ │   UI     │  │
│  └─────────┘ └──────────┘ └────────┘ └────────┘ └──────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Module responsibilities

### Backend (Rust)

- **`registry.rs`** — Finds installed games. Reads Windows Registry (Steam, Epic, GOG, Battle.net, Riot, EA, Ubisoft), scans common install directories, applies heuristics for manual installs. Returns `Vec<InstalledGame>`.
- **`manifest.rs`** — Downloads and caches the [Ludusavi manifest](https://github.com/mtkennerly/ludusavi-manifest) (a JSON file mapping game names to known save paths). Auto-refreshes if older than 7 days.
- **`scanner.rs`** — The heart of the app. Walks user directories (`AppData`, `Documents`, `Saved Games`), classifies folders by name and publisher, cross-references with installed games + manifest, returns `Vec<OrphanedFile>` with category + confidence.
- **`safe_delete.rs`** — The "Golden Rule" implementation. Sends to Recycle Bin by default. Has lock-detection hints for shader cache files. Supports 3 strategies: RecycleBin, BackupFolder, DirectDelete (force).
- **`settings.rs`** — Loads/saves user settings as JSON in the OS app config dir.
- **`progress.rs`** — Helper for emitting `scan-progress` Tauri events to the frontend.
- **`model.rs`** — All shared types: `InstalledGame`, `OrphanedFile`, `ScanReport`, `DeleteStrategy`, `AppSettings`, `Whitelist`, etc.
- **`lib.rs`** — Tauri command exports + app setup + plugin registration.
- **`bin/cli.rs`** — Headless CLI binary for testing and scripting.

### Frontend (HTML/CSS/JS)

- **`index.html`** — Static HTML. 4 main screens: Dashboard, Results, Backup/Migrate, History + 2 modals (Settings, Shortcuts).
- **`main.js`** — All UI logic, Tauri command invocations, state management, event listeners. Vanilla JS, no framework.
- **`styles.css`** — All styling. CSS custom properties for theming. Dark + light theme via `[data-theme="light"]`.

## Key design decisions

### Why Tauri (not Electron)?

A 5 MB installer vs 100 MB+ for Electron. Critical for users who don't want heavy background apps.

### Why no framework on the frontend?

The app is essentially a few screens with a list. React/Vue would add 200 KB+ of framework code for no real benefit. Vanilla JS keeps the binary small and the codebase accessible to anyone who knows basic JS.

### Why the "denied publishers" + "denied game names" lists in `scanner.rs`?

The naive approach (flag every folder in AppData that doesn't match an installed game) gives 50,000+ false positives (Python caches, browser data, dev tools, etc.). The denylist is a hand-curated list of "this is definitely NOT a game save" patterns. It cuts false positives by ~99% while keeping recall high.

The denylist isn't perfect. If we flag a real game folder as orphan, the user can:
- Click "Whitelist" to prevent re-flagging
- Open a GitHub issue so we add the pattern to the denylist

### Why Recycle Bin by default?

The spec's "Golden Rule" — never hard-delete without explicit consent. Recycle Bin is the Windows-native way to make deletion reversible. The `trash` crate wraps the Windows `SHFileOperation` API.

### Why offline license validation for Pro?

No server = no monthly cost = no risk of going bankrupt. The license key is signed with Ed25519, the public key is baked into the app, verification takes ~1ms. Trade-off: harder to revoke a leaked key, but for a $2 product that's fine.

## Performance

- **Quick scan**: ~100-600ms (AppData only)
- **Standard scan**: ~500ms - 2s (AppData + Documents + Saved Games)
- **Deep scan**: ~2-3s (+ Downloads + GPU caches)
- **Directory walking**: bounded by `walkdir` with `max_depth` (6) and `min_depth` (2)
- **Size calculation**: parallel via `walkdir`'s iterator
- **UI render**: debounced, only re-renders when filter/sort/search changes

## Threading

- Tauri commands run on a Tokio runtime
- Long operations (scan, purge) use `tauri::async_runtime::spawn_blocking` to avoid blocking the main thread
- Progress events emitted via `tauri::Emitter` are non-blocking
- The frontend listens to events and updates the UI

## Security

- **No remote code execution** — Tauri doesn't allow eval or remote scripts
- **Whitelist for paths** — only allow known safe paths in IPC commands
- **Tauri capabilities** — explicit permission grants per command
- **CSP** — `csp: null` for now (TODO: add a strict CSP for production)
- **Pro license** — Ed25519 signature verification, no network check
