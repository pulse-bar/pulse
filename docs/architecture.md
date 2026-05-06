# Architecture

> Long-form take on the layout. For a quick reference see
> [CLAUDE.md](../CLAUDE.md).

## Directory shape

```
crates/        engine — Cargo workspace members
apps/          end-user applications (desktop today; daemon, cli later)
extensions/    third-party / out-of-tree plugin crates
packages/      JS/TS shared packages
docs/          long-form documentation
```

The flat shape comes from the openclaw model: `crates/`, `apps/`,
`extensions/`, `packages/` are siblings — there's no nesting like
`apps/<x>/src-tauri/src/`. Each apps/desktop subdirectory has a clear
job: `shell/` is Rust + Tauri config, `ui/` is React + Vite.

## Engine crates

### `crates/core`

- `model.rs` — wire types (UsageTotals, TaskSnapshot, ActiveTask,
  Settings, AttributionOutcome). Mirror in `packages/types/src/types.ts`.
- `turn.rs` — provider-neutral `ParsedTurn`.
- `storage.rs` — SQLite schema + queries (`Db` wraps an r2d2 pool).
  Tables: `sessions`, `turns`, `settings`, `meta`. WAL mode, single
  upsert path keyed on `message_id`.
- `state.rs` — `AppState` owns the 5-hour rolling window.
- `pricing.rs` — USD-per-1M-tokens table.
- `time.rs` — Monday-00:00-UTC weekly boundary, 5h session window,
  90s idle threshold.

No FS, no network, no Tauri. Pure domain.

### `crates/attribution`

```rust
trait AttributionProvider {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;            // higher = runs earlier
    fn try_attribute(...) -> Option<AttributionOutcome>;
}
```

Built-ins:
- `git-branch` (priority 90) — regex against `gitBranch`.
- `cwd` (priority 50) — regex against working directory.

`Registry` runs providers in priority order; first `Some` wins. Adding
a provider is one file plus one `register` line.

### `crates/ingest`

```rust
trait IngestProvider {
    fn name(&self) -> &'static str;
    fn watch_roots(&self) -> Vec<PathBuf>;
    fn matches(&self, path: &Path) -> bool;     // default: *.jsonl
    fn parse_line(&self, line: &str) -> Result<Option<ParsedTurn>, ParseError>;
}
```

Stateless. The watcher does I/O and offset bookkeeping; providers just
know *where to look* and *how to interpret one line*.

### `crates/watcher`

The only crate that touches the live filesystem.

```rust
let watcher = Watcher::new(state, ingest_registry, attribution_registry);
let events = watcher.subscribe();
let handle = watcher.run().await;
// ...
handle.shutdown().await;
```

- **Initial scan** — synchronous; walks every provider's `watch_roots`,
  ingests files in priority order.
- **Live loop** — `notify-debouncer-full` on its own thread, bridged to
  tokio via mpsc. Each tick batches paths, dedupes, and fans out
  concurrent `ingest_path` tasks.
- **Per-path serialisation** — `HashMap<PathBuf, Arc<tokio::Mutex>>`
  prevents two concurrent reads racing on the same offset.
- **Rotation** — `total_size < start_offset` → re-read from byte zero,
  emit `WatcherEvent::FileRotated`.
- **Events** — `WatcherEvent` over a `tokio::broadcast`. Decoupled from
  the UI; future CLI / daemon / test harness all subscribe the same way.

## `apps/desktop`

Pure glue.

- `shell/` — Rust:
  - `lib.rs` — Tauri builder, plugins, runtime setup.
  - `state.rs` — owns `Arc<AppState>` + `Arc<Watcher>`.
  - `bridge.rs` — re-emits `WatcherEvent` as Tauri IPC events.
  - `commands.rs` — IPC handlers (one-liners over the engine crates).
  - `tray.rs` — system tray icon + menu.
- `ui/` — React + Vite:
  - `screens/` — Popover, Dashboard, Settings, Onboarding.
  - `components/` — TrayBar, ToastHost, PulseLogo.
  - `hooks/` — useActiveTask, useDashboard, useSettings.
  - `lib/tauri.ts` — typed IPC wrappers.

## `extensions/`

Future plugin homes. Each subdirectory is a Cargo crate auto-picked up
by the workspace glob `extensions/*`. Three patterns:

```
extensions/attribution-jira/        → impl AttributionProvider
extensions/ingest-codex/            → impl IngestProvider
extensions/notification-slack/      → subscribes to WatcherEvent broadcast
```

The desktop shell registers them at startup. See
[`plugins.md`](./plugins.md).

## Why this shape pays off

- **Plugin-shaped, not framework-shaped.** New providers are new files
  / new crates — no abstraction-creep inside existing modules.
- **Headless mode is free.** A `apps/daemon` binary is `Watcher::new(...).run()`
  with zero UI dependency.
- **Multi-app ready.** `apps/cli`, mobile shells, etc. all reuse the
  same engine crates.
- **Testing is unblocked.** Pure-domain crates and trait providers test
  in isolation.
