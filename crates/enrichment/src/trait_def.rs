use async_trait::async_trait;
use pulse_core::{Settings, TaskMetadata};

use crate::error::EnrichmentResult;

#[async_trait]
pub trait TaskEnricher: Send + Sync {
    fn name(&self) -> &'static str;

    // Whether this enricher is willing to handle the given task_id.
    // Used for routing: e.g. Jira matches `PROJ-123`, Linear matches `ENG-7`.
    fn matches(&self, task_id: &str, settings: &Settings) -> bool;

    // Whether the enricher is currently configured to run (e.g. has a base URL
    // and credentials). Returning false makes the daemon skip it without error.
    fn is_configured(&self, settings: &Settings) -> bool;

    async fn enrich(&self, task_id: &str, settings: &Settings) -> EnrichmentResult<TaskMetadata>;

    // Optional liveness/connectivity check, used by Settings → Test connection.
    async fn test(&self, _settings: &Settings) -> EnrichmentResult<()> {
        Ok(())
    }
}
