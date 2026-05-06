# Extensions

Provider plugins as separate Cargo crates. Each subdirectory is its own
crate; the workspace glob `extensions/*` picks them up automatically.

```
extensions/
├── attribution-jira/        # Cargo crate
├── attribution-linear/
├── ingest-codex/
└── ingest-gemini/
```

An extension crate depends on the trait it implements:

```toml
# extensions/attribution-jira/Cargo.toml
[dependencies]
pulse-core        = { workspace = true }
pulse-attribution = { workspace = true }
```

Then re-export the provider so the desktop app (or any other binary)
can register it. See [`docs/plugins.md`](../docs/plugins.md) for full
recipes.
