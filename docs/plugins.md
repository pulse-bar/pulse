# Plugins

Pulse has two extension points: **attribution** (resolve a turn to a
task) and **ingest** (read transcripts from a new source). Plugins
live in `extensions/` as separate crates and are picked up by the
workspace glob `extensions/*`.

## Built-ins vs. extensions

- **Built-ins** ship inside engine crates:
  `crates/attribution/src/providers/git.rs`,
  `crates/ingest/src/providers/claude_code.rs`, etc. Use this for
  plugins everyone wants by default.
- **Extensions** live under `extensions/<name>/` as their own Cargo
  crate. Use this for opt-in providers, third-party integrations, or
  anything you might want to ship behind a feature flag.

The trait surface is the same in both cases.

## Attribution plugin (extension)

```bash
mkdir -p extensions/attribution-linear/src
```

```toml
# extensions/attribution-linear/Cargo.toml
[package]
name        = "pulse-ext-attribution-linear"
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
authors.workspace      = true
repository.workspace   = true

[dependencies]
pulse-core        = { workspace = true }
pulse-attribution = { workspace = true }
regex             = { workspace = true }
```

```rust
// extensions/attribution-linear/src/lib.rs
use pulse_attribution::AttributionProvider;
use pulse_core::{AttributionConfidence, AttributionOutcome, ParsedTurn, Settings};

#[derive(Clone, Copy)]
pub struct LinearProvider;

impl AttributionProvider for LinearProvider {
    fn name(&self) -> &'static str { "linear" }
    fn priority(&self) -> i32 { 80 }   // runs after git-branch (90)

    fn try_attribute(&self, turn: &ParsedTurn, _: &Settings) -> Option<AttributionOutcome> {
        let branch = turn.branch.as_deref()?;
        let rx = regex::Regex::new(r"(?i)([A-Z]{2,4}-\d+)").ok()?;
        let id = rx.captures(branch)?.get(1)?.as_str().to_uppercase();
        Some(AttributionOutcome {
            task_id: Some(id),
            confidence: AttributionConfidence::High,
            score: 0.9,
        })
    }
}
```

Register it in the desktop shell (or any consumer):

```rust
// apps/desktop/shell/src/state.rs
let attribution = AttributionRegistry::with_defaults();
attribution.register(Arc::new(pulse_ext_attribution_linear::LinearProvider));
```

## Ingest plugin (extension)

```rust
// extensions/ingest-codex/src/lib.rs
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use pulse_core::ParsedTurn;
use pulse_ingest::{IngestProvider, ParseError};

pub const PROVIDER_NAME: &str = "codex";

#[derive(Clone, Copy)]
pub struct CodexProvider;

impl IngestProvider for CodexProvider {
    fn name(&self) -> &'static str { PROVIDER_NAME }

    fn watch_roots(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Some(home) = dirs::home_dir() {
            out.push(home.join(".codex").join("sessions"));
        }
        out
    }

    fn parse_line(&self, _line: &str) -> Result<Option<ParsedTurn>, ParseError> {
        // Map Codex's record shape into ParsedTurn. None for non-billable rows.
        Ok(None)
    }
}
```

Register it next to the default Claude Code provider:

```rust
let ingest = IngestRegistry::with_defaults();
ingest.register(Arc::new(pulse_ext_ingest_codex::CodexProvider));
```

The watcher will:
- Discover existing transcripts on next start.
- Watch the new directory for live changes.
- Run every line through your `parse_line` and the attribution registry.
- Persist alongside Claude Code data; rollups segment by `provider`.

## Patterns to follow

- **Stateless providers.** No mutable globals. Caches go behind a
  `OnceCell` keyed on a stable input.
- **Don't block the watcher.** Network calls in `try_attribute` will
  serialise the entire pipeline. For enrichment (e.g. resolving a
  Jira title), do it in a background task that updates `task_name`
  via a separate command.
- **Confidence calibration.** `high ≥ 0.85`, `medium` in `0.6..0.85`,
  `low` below. The UI uses these for the badge colour.
- **Priority is `i32`.** Built-ins use 0–100; community plugins can use
  higher numbers if they need to outrank built-ins.

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_branch() {
        let mut t = ParsedTurn { /* ... */ };
        t.branch = Some("feat/PROJ-123-add-foo".into());
        let out = LinearProvider.try_attribute(&t, &Settings::default()).unwrap();
        assert_eq!(out.task_id.as_deref(), Some("PROJ-123"));
    }
}
```

`cargo test -p pulse-ext-attribution-linear`.
