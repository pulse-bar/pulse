// Trait + registry. New providers go in `providers/` and register in `with_defaults`.

pub mod providers;

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use pulse_core::ParsedTurn;

pub trait IngestProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn watch_roots(&self) -> Vec<PathBuf>;
    fn matches(&self, path: &std::path::Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("jsonl")
    }
    fn parse_line(&self, line: &str) -> Result<Option<ParsedTurn>, ParseError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("malformed json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

#[derive(Clone, Default)]
pub struct Registry {
    providers: Arc<RwLock<Vec<Arc<dyn IngestProvider>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let r = Self::new();
        r.register(Arc::new(providers::claude_code::ClaudeCodeProvider));
        r
    }

    pub fn register(&self, provider: Arc<dyn IngestProvider>) {
        self.providers.write().push(provider);
    }

    pub fn providers(&self) -> Vec<Arc<dyn IngestProvider>> {
        self.providers.read().clone()
    }

    pub fn provider_for(&self, path: &std::path::Path) -> Option<Arc<dyn IngestProvider>> {
        self.providers
            .read()
            .iter()
            .find(|p| p.matches(path))
            .cloned()
    }
}
