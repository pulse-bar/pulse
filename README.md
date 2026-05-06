# Pulse

> AI usage tracker for developer teams. Tray-resident, JSONL-driven, Jira-aware.

Pulse sits in your menu bar and tracks AI usage per developer per task.
It reads the session transcripts Claude Code already writes to disk,
attributes usage to Jira tasks via your active git branch, and surfaces
everything in a compact always-visible bar — zero behaviour change.

```
~/.claude/projects/<project>/<session>.jsonl
        │
        ▼
   watcher → ingest → attribution → storage → UI
```

---

## Get started

```bash
git clone https://github.com/pulse-bar/pulse.git
cd pulse
pnpm install
pnpm dev
```

You'll need **Node ≥ 20**, **pnpm ≥ 9**, and **Rust stable**. For the
full per-OS setup (Xcode CLT / WebKitGTK / WebView2, etc.) see
[`docs/install.md`](./docs/install.md).

---

## Layout

```
crates/         core · attribution · ingest · watcher
apps/desktop/   shell (Rust) · ui (React)
extensions/     provider plugins (drop new crates here)
packages/types/ TS wire-format
docs/           architecture · plugins · watcher · platform · install
```

---

## Documentation

| What                | Where |
| ------------------- | ----- |
| Install per OS      | [`docs/install.md`](./docs/install.md) |
| Architecture        | [`docs/architecture.md`](./docs/architecture.md) |
| Adding a plugin     | [`docs/plugins.md`](./docs/plugins.md) |
| The watcher daemon  | [`docs/watcher.md`](./docs/watcher.md) |
| Per-OS notes        | [`docs/platform.md`](./docs/platform.md) |
| Vision / roadmap    | [`VISION.md`](./VISION.md) |
| Contributing        | [`CONTRIBUTING.md`](./CONTRIBUTING.md) |
| Security policy     | [`SECURITY.md`](./SECURITY.md) |
| Changelog           | [`CHANGELOG.md`](./CHANGELOG.md) |
| AI-session brief    | [`CLAUDE.md`](./CLAUDE.md) |

---

## License

MIT — see [LICENSE](./LICENSE).
