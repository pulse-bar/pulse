# Platform notes

Pulse is cross-platform — one codebase, one binary per OS. This doc
captures the bits that are platform-specific so contributors don't
accidentally break one OS while fixing another.

## Supported targets

| OS       | Min version            | Frontend WebView   | FS watcher backend       |
| -------- | ---------------------- | ------------------ | ------------------------ |
| macOS    | 11 Big Sur             | WKWebView          | FSEvents                 |
| Linux    | Ubuntu 22.04 / Fedora 38 (any glibc-2.35+) | WebKitGTK 4.1     | inotify                  |
| Windows  | 10 1809 (with WebView2)| Microsoft WebView2 | ReadDirectoryChangesW    |

The watcher abstraction is `notify-debouncer-full` with
`RecommendedWatcher`, which auto-picks the right backend at compile time
for each OS.

## Platform-specific behaviour

### macOS

- **Tray icon**: `icon_as_template: true` makes the icon render as a
  monochrome glyph that adapts to the menu bar (light/dark/Big Sur
  blur). The PNG must be alpha-only; non-transparent pixels become the
  outline.
- **Popover transparency**: enabled via `macOSPrivateApi: true` in
  `tauri.conf.json` and the `macos-private-api` Cargo feature on the
  Tauri crate. ⚠ This uses Apple's private APIs and **prevents App
  Store distribution**. We currently ship via `.dmg` only. If that
  changes, drop transparency and use a solid popover background
  instead.
- **Autostart**: implemented via `LaunchAgent`
  (`~/Library/LaunchAgents/dev.pulse.app.plist`) — the
  `MacosLauncher::LaunchAgent` parameter to the autostart plugin.
- **App data dir**: `~/Library/Application Support/dev.pulse.app/`
  (resolved via `Tauri::path().app_data_dir()`).
- **Claude Code transcripts**: `~/.claude/projects/<encoded-cwd>/`.

### Linux

- **Compositor required for transparency**: most modern DEs (GNOME,
  KDE, Sway) include one. On a bare X11 setup the popover falls back
  to its solid CSS background — visually still correct.
- **Tray icon**: requires `libayatana-appindicator3` (or
  `libappindicator3` on older distros). The icon is rendered as-is —
  template mode is ignored, so the white-on-transparent PNG shows on
  dark top bars correctly. On light DEs it may be hard to see; swap in
  a darker icon if needed.
- **Autostart**: writes a `.desktop` file under
  `~/.config/autostart/dev.pulse.app.desktop`.
- **App data dir**: `~/.local/share/dev.pulse.app/`.
- **Claude Code transcripts**: `~/.claude/projects/<encoded-cwd>/`.
- **System deps** (Debian/Ubuntu):
  ```
  libwebkit2gtk-4.1-dev
  libayatana-appindicator3-dev
  librsvg2-dev
  build-essential
  libssl-dev
  ```

### Windows

- **WebView2**: pre-installed on Windows 11. Windows 10 may require
  installing the Evergreen Bootstrapper from
  https://developer.microsoft.com/microsoft-edge/webview2/.
- **Build prerequisites**: Visual Studio 2022 Build Tools with the
  "Desktop development with C++" workload (provides MSVC + WinSDK).
- **Tray icon**: rendered as-is; template mode is ignored. The
  white-alpha icon shows correctly against the dark system tray.
- **Autostart**: writes a value under
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **App data dir**: `%APPDATA%\dev.pulse.app\`.
- **Claude Code transcripts**: `%USERPROFILE%\.claude\projects\<encoded-cwd>\`.
- **Console window**: hidden in release via
  `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
  in `apps/desktop/shell/src/main.rs`. Debug builds keep the console
  attached so `tracing` output is visible.
- **Path separators**: SQLite stores file paths verbatim
  (`to_string_lossy()`), so the `\` separator is preserved through
  rotations.

## What to test on each OS before releasing

```
[ ] First-run onboarding wizard appears
[ ] Tray icon visible in menu bar / system tray
[ ] Left-click tray → popover opens, anchored to icon
[ ] Right-click tray → menu shows Dashboard / Settings / Quit
[ ] Dashboard auto-opens and renders task table
[ ] Watcher picks up a fresh `~/.claude/projects/.../*.jsonl` write within ~250ms
[ ] Restart the app — meters resume from persisted state, no re-scan
[ ] Settings → "Re-scan all transcripts" rebuilds rollups
[ ] Closing all windows hides them (does not quit)
[ ] Quit from tray menu fully exits
```

## Conditional compilation cheatsheet

If you genuinely need OS-specific code (try not to — most things should
go through abstractions like `Tauri::path()` or `dirs::*`):

```rust
#[cfg(target_os = "macos")]
fn macos_only() { /* ... */ }

#[cfg(target_os = "linux")]
fn linux_only() { /* ... */ }

#[cfg(target_os = "windows")]
fn windows_only() { /* ... */ }

#[cfg(not(target_os = "macos"))]
fn everything_except_macos() { /* ... */ }
```

For Cargo features (e.g. enabling `macos-private-api` only when
relevant), see how the Tauri dep is declared in
`apps/desktop/shell/Cargo.toml`.

## CI matrix

`.github/workflows/ci.yml` runs `cargo check --workspace` and
`cargo test --workspace` on `ubuntu-latest`, `macos-latest`, and
`windows-latest` for every PR. The release workflow bundles per-OS
artifacts. If a PR breaks Linux or Windows, CI will catch it before
merge — but please run `cargo check` locally on at least your own OS
first.
