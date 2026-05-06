# Changelog

All notable changes to Pulse are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning
follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- (nothing yet)

## [0.1.0] — 2026-05-06

Initial public scaffold.

### Added
- Cargo workspace with four engine crates: `core`, `attribution`,
  `ingest`, `watcher`.
- Trait-based plugin system for attribution providers
  (`AttributionProvider`) and ingest sources (`IngestProvider`).
- Built-in attribution providers: `git-branch`, `cwd`.
- Built-in ingest provider: `claude-code`.
- File-watcher daemon with concurrent per-file ingestion, debounced
  burst handling, file-rotation detection, and a structured event
  broadcast channel.
- Tauri v2 desktop app (`apps/desktop`) with tray icon, popover,
  dashboard, settings, and onboarding wizard.
- React UI ported from the canonical Pulse mock — pulse-purple theme,
  IBM Plex fonts, all four screens.
- Local SQLite storage with WAL mode, byte-offset resume, and
  `message_id` deduplication.
- Pricing tables for Claude Opus / Sonnet / Haiku families.
- Cross-platform builds: macOS, Linux, Windows.
- Documentation: `README.md`, `CLAUDE.md`, `CONTRIBUTING.md`,
  `VISION.md`, `SECURITY.md`, plus `docs/architecture.md`,
  `docs/plugins.md`, `docs/watcher.md`, `docs/platform.md`,
  `docs/install.md`.
- GitHub Actions CI (`typecheck` + `cargo check` + `cargo test` on
  Linux/macOS/Windows; bundle-on-tag release workflow).

[Unreleased]: https://github.com/pulse-bar/pulse/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pulse-bar/pulse/releases/tag/v0.1.0
