use std::sync::Arc;

use parking_lot::RwLock;
use pulse_core::Settings;

use crate::trait_def::TaskEnricher;

#[derive(Clone, Default)]
pub struct Registry {
    enrichers: Arc<RwLock<Vec<Arc<dyn TaskEnricher>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, enricher: Arc<dyn TaskEnricher>) {
        self.enrichers.write().push(enricher);
    }

    pub fn enrichers(&self) -> Vec<Arc<dyn TaskEnricher>> {
        self.enrichers.read().clone()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.enrichers.read().iter().map(|e| e.name()).collect()
    }

    // Returns the first enricher whose `matches()` accepts the given
    // task_id. Sites/projects with overlapping keys are routed to whichever
    // was registered first; consumers that need stricter routing should
    // register with that ordering in mind.
    pub fn pick(&self, task_id: &str, settings: &Settings) -> Option<Arc<dyn TaskEnricher>> {
        self.enrichers
            .read()
            .iter()
            .find(|e| e.is_configured(settings) && e.matches(task_id, settings))
            .cloned()
    }
}
