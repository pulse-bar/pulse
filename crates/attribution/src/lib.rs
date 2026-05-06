// Trait + registry. New providers go in `providers/` and register in `with_defaults`.

pub mod providers;

use std::sync::Arc;

use parking_lot::RwLock;
use pulse_core::{AttributionOutcome, ParsedTurn, Settings};

pub trait AttributionProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn try_attribute(&self, turn: &ParsedTurn, settings: &Settings) -> Option<AttributionOutcome>;
}

#[derive(Clone, Default)]
pub struct Registry {
    providers: Arc<RwLock<Vec<Arc<dyn AttributionProvider>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let r = Self::new();
        r.register(Arc::new(providers::git::GitBranchProvider));
        r.register(Arc::new(providers::cwd::CwdProvider));
        r
    }

    pub fn register(&self, provider: Arc<dyn AttributionProvider>) {
        let mut list = self.providers.write();
        list.push(provider);
        list.sort_by_key(|p| -p.priority());
    }

    pub fn resolve(&self, turn: &ParsedTurn, settings: &Settings) -> AttributionOutcome {
        for p in self.providers.read().iter() {
            if let Some(out) = p.try_attribute(turn, settings) {
                return out;
            }
        }
        AttributionOutcome::unattributed()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.providers.read().iter().map(|p| p.name()).collect()
    }
}
