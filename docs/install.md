# Install

You need **Node ≥ 20**, **pnpm ≥ 9**, and **Rust stable** on every OS.
The first `pnpm dev` compiles Rust deps from source (~3–5 min).
Subsequent runs are cached.

## macOS

```bash
xcode-select --install
brew install node@20 && corepack enable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

git clone https://github.com/pulse-bar/pulse.git
cd pulse && pnpm install && pnpm dev
```

Min version: macOS 11 Big Sur.

## Linux — Debian / Ubuntu 22.04+

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  build-essential curl libssl-dev pkg-config

curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs && corepack enable

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

git clone https://github.com/pulse-bar/pulse.git
cd pulse && pnpm install && pnpm dev
```

## Linux — Fedora / RHEL

```bash
sudo dnf install -y webkit2gtk4.1-devel libappindicator-gtk3-devel \
  librsvg2-devel openssl-devel curl wget file gcc gcc-c++ make
```

Then proceed with the Node / pnpm / Rust / clone steps from the Debian
section above.

## Linux — Arch

```bash
sudo pacman -Syu --needed webkit2gtk-4.1 libappindicator-gtk3 \
  librsvg base-devel curl wget file openssl
```

Then proceed as above.

## Windows

In an Administrator PowerShell:

```powershell
winget install -e --id Microsoft.VisualStudio.2022.BuildTools `
  --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --quiet --norestart"
winget install -e --id Microsoft.EdgeWebView2Runtime
winget install -e --id OpenJS.NodeJS.LTS
winget install -e --id Rustlang.Rustup
corepack enable
rustup default stable

git clone https://github.com/pulse-bar/pulse.git
Set-Location pulse
pnpm install ; pnpm dev
```

Min version: Windows 10 1809 (with WebView2).

## Daily commands

```bash
pnpm dev                  # hot-reload UI + Rust shell
pnpm build                # release bundle (.dmg / .AppImage / .msi)
pnpm typecheck            # TypeScript gate
cargo check --workspace   # Rust gate
cargo test  --workspace   # tests
```

## Troubleshooting

**`error: linker not found`** — install your platform's C toolchain:
Xcode CLT (macOS), `build-essential` (Linux), MSVC Build Tools (Windows).

**`webkit2gtk-4.1-dev` not found on Linux** — older Ubuntu (20.04) only
ships `4.0`. Upgrade to 22.04+, or pin Tauri's `webkit2gtk` feature to
`4_0` (not currently supported in this branch).

**`Couldn't recognize the current folder as a Tauri project`** — run
`pnpm dev` from the repo root, which routes through
`pnpm --filter @pulse/desktop tauri:dev` so the CLI runs from
`apps/desktop/` where it can see `shell/tauri.conf.json` as a subfolder.

**Dashboard window doesn't open on macOS Sonoma+** — verify the
`macOSPrivateApi` feature is on (default). Without it, transparent
windows fail to draw under Sonoma's strict mode.

**Tray icon missing on Linux** — install `libayatana-appindicator3`
and confirm your DE supports StatusNotifierItem (GNOME needs the
AppIndicator extension).

**`401 Unauthorized` from a corporate npm registry** — your global
`~/.npmrc` is routing installs through a private registry. Pulse ships
a project-local `.npmrc` that overrides the registry to public npm; if
you've removed it, restore it.
