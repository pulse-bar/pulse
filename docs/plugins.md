# Plugins

Pulse has three extension points. Each is a trait + a registry; adding a
provider is one new file (or one new crate) plus one registration line.

| Trait | Lives in | What it does | Built-ins | Extensions today |
| --- | --- | --- | --- | --- |
| `IngestProvider`      | `crates/ingest`      | Discover + parse AI-tool transcripts                        | Claude Code           | — |
| `AttributionProvider` | `crates/attribution` | Resolve a `ParsedTurn` to a task ID                         | git-branch, cwd       | — |
| `TaskEnricher`        | `crates/enrichment`  | Resolve a task ID into rich metadata (title, status, URL …) | none                  | `extensions/attribution-jira` |

Built-ins ship inside engine crates. Out-of-tree plugins live under
`extensions/` as their own Cargo crate; the workspace glob
`extensions/*` picks them up automatically.

---

## Add an attribution provider

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

Register it in the desktop shell (`apps/desktop/shell/src/state.rs`):

```rust
let attribution = AttributionRegistry::with_defaults();
attribution.register(Arc::new(pulse_ext_attribution_linear::LinearProvider));
```

---

## Add an ingest provider

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

---

## Add a task enricher (Jira / Linear / GitHub Issues)

A `TaskEnricher` runs **out of band** of the watcher — it polls SQLite
for unenriched task IDs on its own schedule, calls a remote API, and
upserts metadata. The watcher pipeline never blocks on network.

### Trait

```rust
#[async_trait]
pub trait TaskEnricher: Send + Sync {
    fn name(&self) -> &'static str;
    fn matches(&self, task_id: &str, settings: &Settings) -> bool;
    fn is_configured(&self, settings: &Settings) -> bool;
    async fn enrich(&self, task_id: &str, settings: &Settings) -> EnrichmentResult<TaskMetadata>;
    async fn test(&self, settings: &Settings) -> EnrichmentResult<()> { Ok(()) }
}
```

### Recipe — Linear enricher

```bash
mkdir -p extensions/enrichment-linear/src
```

```toml
[dependencies]
pulse-core       = { workspace = true }
pulse-enrichment = { workspace = true }
async-trait      = { workspace = true }
chrono           = { workspace = true }
keyring          = "3"
reqwest          = { version = "0.12", features = ["json", "rustls-tls"] }
serde            = { workspace = true }
serde_json       = { workspace = true }
tokio            = { workspace = true }
```

```rust
// extensions/enrichment-linear/src/lib.rs
use async_trait::async_trait;
use pulse_core::{Settings, TaskMetadata};
use pulse_enrichment::{EnrichmentError, EnrichmentResult, TaskEnricher};

pub struct LinearEnricher { /* http client, keychain helpers, … */ }

#[async_trait]
impl TaskEnricher for LinearEnricher {
    fn name(&self) -> &'static str { "linear" }

    fn matches(&self, task_id: &str, _settings: &Settings) -> bool {
        // route by your own ID format
        task_id.split_once('-').map(|(p, _)| p.len() <= 4).unwrap_or(false)
    }

    fn is_configured(&self, settings: &Settings) -> bool {
        // walk settings.linear.workspaces or however you model it
        true
    }

    async fn enrich(&self, task_id: &str, _settings: &Settings) -> EnrichmentResult<TaskMetadata> {
        // POST to https://api.linear.app/graphql
        // map response into TaskMetadata
        Err(EnrichmentError::Other("not implemented".into()))
    }
}
```

Register in the desktop shell:

```rust
let enrichment = EnrichmentRegistry::new();
enrichment.register(Arc::new(JiraEnricher::new()));
enrichment.register(Arc::new(pulse_ext_enrichment_linear::LinearEnricher::new()));
```

### Multi-instance / multi-team config

The Jira reference impl shows the canonical shape:

- `Settings.jira.sites: Vec<JiraSite>` — N sites, each with project-key list, base URL, auth kind.
- Routing by project-key prefix (`PROJ-` → site A, `WEB-` → site B).
- A site with empty `project_keys` acts as a fallback for unrecognised prefixes.
- Tokens stored in the OS keychain via `keyring-rs`, keyed on `site.id`. Settings only stores references — no plaintext credentials.

Mirror that shape for Linear workspaces, GitHub Issues organisations, etc.

### Patterns to follow

- **Stateless providers.** No mutable globals. Caches go behind a
  `OnceCell` keyed on a stable input.
- **Don't block the watcher.** Network calls in `try_attribute` would
  serialise the whole pipeline. Enrichers fix this by running in their
  own daemon — keep that boundary clean.
- **Confidence calibration.** `high ≥ 0.85`, `medium` in `0.6..0.85`,
  `low` below. The UI uses these for the badge colour.
- **Priority is `i32`.** Built-ins use 0–100; community plugins can use
  higher numbers if they need to outrank built-ins.
- **Graceful degradation.** When the API is unreachable, return a
  recoverable `EnrichmentError`. The daemon will retry on the next tick.
- **Honour rate-limit hints.** Return `EnrichmentError::RateLimited`
  with the server's `Retry-After` value; the daemon will sleep before
  resuming.

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn jira_routes_by_project_key() {
        // …
    }
}
```

`cargo test -p pulse-ext-attribution-jira`.
