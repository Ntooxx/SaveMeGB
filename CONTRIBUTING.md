# Contributing to SaveMeGB

Thanks for your interest in contributing! 🎉

SaveMeGB is open source under MIT. That means anyone can read, modify, and distribute the code. We welcome contributions of all kinds.

## Ways to contribute

- 🐛 **Report bugs** — open an issue with reproduction steps
- 💡 **Suggest features** — open a feature request issue
- 📝 **Improve docs** — typo fixes, clarifications, new examples
- 🧪 **Add launcher support** — see "Adding a new launcher" below
- 🔧 **Submit a PR** — code changes, bug fixes, refactors
- 🌍 **Translate** — help localize the UI to your language
- 📣 **Spread the word** — star the repo, share on social media, write a blog post

## Adding a new launcher

The engine is designed to be extensible. To add support for a new game launcher:

1. Open `src-tauri/src/registry.rs`
2. Add a new `scan_<launcher>()` function that returns `Result<Vec<InstalledGame>>`
3. Wire it up in `scan_all()` (the list of launchers to scan)
4. Add test cases in `src-tauri/tests/engine.rs`
5. Add the launcher name to the denylist / allowed-list in `src-tauri/src/scanner.rs` if it needs special handling

Pull request checklist:
- [ ] Function follows the existing pattern (returns Result, logs warnings)
- [ ] Handles missing registry keys / directories gracefully
- [ ] New code paths have at least one test
- [ ] No new dependencies unless discussed in an issue first

## Development setup

### Prerequisites

- **Node.js 18+** and **npm**
- **Rust 1.75+** (stable)
- **Windows 10+**, **macOS 11+**, or **Linux** with WebKitGTK
- **Tauri 2 prerequisites** for your platform — see https://tauri.app/start/prerequisites/

### First run

```bash
git clone https://github.com/Ntooxx/SaveMeGB.git
cd SaveMeGB
npm install
npm run tauri dev
```

The app will launch in dev mode with hot-reload.

### Run tests

```bash
# Rust engine tests
cargo test --manifest-path src-tauri/Cargo.toml --tests

# All in one
npm test
```

### Code style

- **Rust**: standard `rustfmt` and `clippy`
- **JS**: 2-space indent, single quotes, no semicolons (or yes, both are fine, just be consistent)
- **CSS**: BEM-ish naming, prefer CSS variables for colors

### Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Riot Games launcher support
fix: shader cache delete failed on locked files
docs: update README with new screenshots
chore: bump dependencies
```

## Pull request process

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Add tests if applicable
4. Run the test suite — all must pass
5. Open a PR with a clear description
6. Wait for review (usually within a week)

## Code of conduct

Be kind. We're all just trying to make disk space cheaper. See `CODE_OF_CONDUCT.md`.

## Questions?

Open a discussion or reach out via the issues. We're friendly.
