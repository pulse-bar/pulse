use once_cell::sync::Lazy;
use parking_lot::RwLock;
use regex::Regex;

use pulse_core::{AttributionConfidence, AttributionOutcome, ParsedTurn, Settings};

use crate::AttributionProvider;

// Pattern is user-tunable, so cache compiled regex and invalidate on change.
static COMPILED: Lazy<RwLock<(String, Regex)>> = Lazy::new(|| {
    let pattern = r"(?i)([A-Z][A-Z0-9]+-\d+)".to_string();
    let rx = Regex::new(&pattern).expect("default regex");
    RwLock::new((pattern, rx))
});

#[derive(Clone, Copy)]
pub struct GitBranchProvider;

impl AttributionProvider for GitBranchProvider {
    fn name(&self) -> &'static str { "git-branch" }
    fn priority(&self) -> i32 { 90 }

    fn try_attribute(&self, turn: &ParsedTurn, settings: &Settings) -> Option<AttributionOutcome> {
        let branch = turn.branch.as_deref()?;
        if branch.is_empty() || branch == "HEAD" {
            return None;
        }
        let rx = compiled(&settings.branch_regex);
        let captured = rx.captures(branch)?.get(1)?.as_str().to_uppercase();

        // Foreign keys (outside any configured Jira-site project list) downgrade to medium.
        let configured_keys: Vec<String> = settings
            .jira
            .sites
            .iter()
            .filter(|s| s.enabled)
            .flat_map(|s| s.project_keys.iter().cloned())
            .collect();
        if !configured_keys.is_empty() {
            let prefix_ok = configured_keys
                .iter()
                .any(|k| captured.starts_with(&format!("{}-", k.to_uppercase())));
            if !prefix_ok {
                return Some(AttributionOutcome {
                    task_id: Some(captured),
                    confidence: AttributionConfidence::Medium,
                    score: 0.7,
                });
            }
        }

        Some(AttributionOutcome {
            task_id: Some(captured),
            confidence: AttributionConfidence::High,
            score: 0.92,
        })
    }
}

fn compiled(pattern: &str) -> Regex {
    {
        let cache = COMPILED.read();
        if cache.0 == pattern {
            return cache.1.clone();
        }
    }
    let mut cache = COMPILED.write();
    if cache.0 != pattern {
        match Regex::new(pattern) {
            Ok(rx) => *cache = (pattern.to_string(), rx),
            Err(err) => {
                tracing::warn!("invalid branch_regex {pattern:?}: {err}");
                *cache = (
                    pattern.to_string(),
                    Regex::new(r"(?i)([A-Z][A-Z0-9]+-\d+)").unwrap(),
                );
            }
        }
    }
    cache.1.clone()
}
