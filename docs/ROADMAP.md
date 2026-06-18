# Roadmap

This is the public roadmap for SaveMeGB. It changes based on user feedback, technical surprises, and free time.

**Legend**: ✅ done · 🚧 in progress · 📅 planned · ❓ under consideration

## v0.1.0 — Open source release ✅

- 9 launchers supported: Steam, Epic, Xbox, GOG, Battle.net, Riot, EA, Ubisoft, manual
- 3 scan modes: quick, standard, deep
- 7 categories: saves, cache, shaders, logs, crashes, settings, backups
- Library + Insights screens
- Recycle Bin, Backup folder, Force Delete strategies
- Whitelist, export, dark/light themes

## v0.2.0 — Pro tier 🚧

- License key activation (offline Ed25519 signature)
- Scheduled auto-cleanup (weekly / monthly)
- Cloud backup of saves before delete (OneDrive / Dropbox sync folder)
- Unlimited history (free tier limited to 30 events)
- Custom save paths (add paths the scanner doesn't know about)
- Pricing: $2 lifetime via Gumroad

## v0.3.0 — Polish

- More accurate orphan detection (machine learning classifier?)
- Per-publisher / per-game filter chips
- Group by game view (multiple orphans for one game)
- Compact list view
- Drag-and-drop to whitelist
- Keyboard arrow navigation
- Right-click context menus
- Scan comparison (this scan vs last scan, side by side)

## v0.4.0 — More launchers

- Bethesda.net launcher
- Amazon Games
- Itch.io
- Manual Steam-family-shared library detection
- Custom path patterns (regex)

## v0.5.0 — Localization 📅

- Multi-language UI (English first, then community-driven translations)
- Localized scan paths (e.g., different AppData names in Chinese Windows)
- Right-to-left support for Arabic / Hebrew

## v0.6.0 — Mac and Linux

- macOS port (using Tauri's macOS support)
- Linux port (AppImage / .deb)
- Handle platform-specific differences (Recycle Bin → Trash, registry → plist)

## v1.0.0 — Microsoft Store release

- Microsoft Store submission
- Code signing certificate
- Auto-update via Tauri's updater plugin
- Polished installer

## Ideas (no commitment)

- Steam Deck detection (handle SteamOS paths)
- Proton save detection (Linux-compat layer for Windows games)
- Cloud sync of whitelist across devices
- Plugin system for community-contributed classifiers
- CLI for scripting: `savemegb-cli scan --mode deep --json | jq`
- Web UI for managing scan history

## How to influence the roadmap

- 👍 React to issues with 👍 on [GitHub](https://github.com/Ntooxx/SaveMeGB/issues) to vote
- Open a [feature request](https://github.com/Ntooxx/SaveMeGB/issues/new?template=feature_request.md) with your use case
- Join [Discussions](https://github.com/Ntooxx/SaveMeGB/discussions) and tell us what you'd use

We do our best to build what users actually want, not what we think is cool.
