# Contributing to Pulse

Short version: keep changes small, keep the code boring, keep the data
local. Look at how an existing plugin is written and copy that shape.

## Setup

See [README.md](./README.md) for per-platform install. Once set up:

```bash
pnpm install
pnpm dev
```

## Workflow

1. Branch named `<type>/<jira>-<slug>` — e.g.
   `feat/PROJ-123-add-linear-attribution`. Pulse's own attribution
   regex resolves this to the ticket automatically.
2. Make the change. One concern per PR.
3. Run the gates locally:
   ```bash
   pnpm typecheck
   cargo check --workspace
   cargo test  --workspace
   ```
4. Open the PR.

## Conventions

- **One concern per file.** Plugins are new files / new crates.
- **No "what" comments.** Names should explain the what; comments are
  for invariants, constraints, workarounds.
- **Wire format is camelCase.** `serde(rename_all = "camelCase")` on
  the Rust side; `packages/types/src/types.ts` matches field-for-field.
- **No telemetry, no remote calls** outside user-configured Jira /
  Linear / etc. integrations. Pulse is local-first.
- **Empty states say so.** Don't render fake bars without data.
- **Workspace-wide deps.** Versions live in the root `Cargo.toml`'s
  `[workspace.dependencies]`.

## Commits

Conventional, lower case:

```
feat(attribution): add linear plugin
fix(watcher): handle truncated jsonl on manual rotation
chore(deps): bump tauri to 2.2.0
docs(plugins): clarify priority ranges
```

## Pull requests

- Title mirrors the commit subject.
- Body has three sections: **What**, **Why**, **Test plan**.
- For UI changes attach before/after screenshots of the affected
  surface (popover, dashboard, settings, onboarding).

## Architecture

[CLAUDE.md](./CLAUDE.md) — single-page reference.
[`docs/architecture.md`](./docs/architecture.md) — long form.
[`docs/plugins.md`](./docs/plugins.md) — extension recipes.
[`docs/watcher.md`](./docs/watcher.md) — concurrency / rotation details.
