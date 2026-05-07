use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use pulse_core::{AppState, EnrichmentState, EnrichmentStatus, TaskMetadata};
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;

use crate::error::EnrichmentError;
use crate::registry::Registry;

#[derive(Debug, Clone)]
pub enum EnrichmentEvent {
    Started,
    Stopped,
    TaskEnriched { metadata: TaskMetadata },
    Error { task_id: Option<String>, message: String },
    StatusChanged { status: EnrichmentStatus },
}

pub struct EnrichmentHandle {
    shutdown: Option<oneshot::Sender<()>>,
    join: JoinHandle<()>,
}

impl EnrichmentHandle {
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = self.join.await;
    }
}

#[derive(Clone)]
pub struct EnrichmentDaemon {
    state: Arc<AppState>,
    registry: Registry,
    events: broadcast::Sender<EnrichmentEvent>,
}

impl EnrichmentDaemon {
    pub fn new(state: Arc<AppState>, registry: Registry) -> Self {
        let (events, _) = broadcast::channel(128);
        Self { state, registry, events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EnrichmentEvent> {
        self.events.subscribe()
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn current_status(&self) -> EnrichmentStatus {
        let settings = self.state.settings();
        let pending = self
            .state
            .db()
            .pending_enrichment_count(settings.enrichment_cache_ttl_secs)
            .unwrap_or(0);
        EnrichmentStatus {
            state: if !settings.enrichment_enabled {
                EnrichmentState::Disabled
            } else if pending > 0 {
                EnrichmentState::Running
            } else {
                EnrichmentState::Idle
            },
            last_run_at: None,
            last_error: None,
            pending_count: pending,
            enrichers: self
                .registry
                .names()
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }

    pub async fn run_once(&self) -> u64 {
        let settings = self.state.settings();
        if !settings.enrichment_enabled {
            return 0;
        }
        let ttl = settings.enrichment_cache_ttl_secs;
        let pending = match self.state.db().unenriched_task_ids(ttl, 64) {
            Ok(ids) => ids,
            Err(err) => {
                let _ = self.events.send(EnrichmentEvent::Error {
                    task_id: None,
                    message: format!("query unenriched: {err}"),
                });
                return 0;
            }
        };

        let mut enriched = 0u64;
        for task_id in pending {
            let Some(enricher) = self.registry.pick(&task_id, &settings) else {
                continue;
            };
            match enricher.enrich(&task_id, &settings).await {
                Ok(meta) => {
                    if let Err(err) = self.state.db().upsert_task_metadata(&meta) {
                        let _ = self.events.send(EnrichmentEvent::Error {
                            task_id: Some(task_id.clone()),
                            message: format!("persist: {err}"),
                        });
                        continue;
                    }
                    let _ = self.events.send(EnrichmentEvent::TaskEnriched {
                        metadata: meta,
                    });
                    enriched += 1;
                }
                Err(EnrichmentError::RateLimited { retry_after_secs }) => {
                    tracing::info!(
                        target: "pulse-enrichment",
                        "{} rate-limited, sleeping {retry_after_secs}s",
                        enricher.name()
                    );
                    tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
                    break;
                }
                Err(err) => {
                    // Persist a stub row so we don't hammer the API for the same
                    // failing ID; refresh-after-TTL still picks it up later.
                    let _ = self.state.db().upsert_task_metadata(&TaskMetadata {
                        task_id: task_id.clone(),
                        enricher: enricher.name().into(),
                        title: None,
                        status: None,
                        assignee: None,
                        url: None,
                        project_key: None,
                        issue_type: None,
                        priority: None,
                        fetched_at: Utc::now(),
                    });
                    let _ = self.events.send(EnrichmentEvent::Error {
                        task_id: Some(task_id.clone()),
                        message: format!("{}: {err}", enricher.name()),
                    });
                }
            }
        }
        enriched
    }

    pub async fn run(self) -> EnrichmentHandle {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let daemon = self.clone();
        let join = tokio::spawn(async move {
            let _ = daemon.events.send(EnrichmentEvent::Started);
            loop {
                let interval = daemon
                    .state
                    .settings()
                    .enrichment_interval_secs
                    .clamp(5, 60 * 60);
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    _ = tokio::time::sleep(Duration::from_secs(interval)) => {
                        let n = daemon.run_once().await;
                        if n > 0 {
                            tracing::debug!(target: "pulse-enrichment", "enriched {n} task(s)");
                        }
                        let _ = daemon.events.send(EnrichmentEvent::StatusChanged {
                            status: daemon.current_status(),
                        });
                    }
                }
            }
            let _ = daemon.events.send(EnrichmentEvent::Stopped);
        });
        EnrichmentHandle { shutdown: Some(shutdown_tx), join }
    }
}
