// File-watcher daemon. Owns the FS watcher and orchestrates ingest →
// attribution → storage. Surfaces every meaningful event on a broadcast
// channel; consumers (Tauri shell, future CLI/daemon) subscribe.

mod ingest_loop;
mod scan;

pub use ingest_loop::WatcherHandle;

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use pulse_attribution::Registry as AttributionRegistry;
use pulse_core::{AppState, AttributionOutcome, ParsedTurn};
use pulse_ingest::Registry as IngestRegistry;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum WatcherEvent {
    InitialScanStarted { roots: Vec<PathBuf> },
    InitialScanProgress { discovered: usize, processed: usize },
    InitialScanComplete { total_turns: u64 },
    TurnIngested {
        turn: ParsedTurn,
        outcome: AttributionOutcome,
        cost_usd: f64,
    },
    FileRotated { path: PathBuf },
    Error { path: Option<PathBuf>, message: String },
}

#[derive(Clone)]
pub struct Watcher {
    state: Arc<AppState>,
    ingest: IngestRegistry,
    attribution: AttributionRegistry,
    events: broadcast::Sender<WatcherEvent>,
    in_flight: Arc<Mutex<std::collections::HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>>,
}

impl Watcher {
    pub fn new(
        state: Arc<AppState>,
        ingest: IngestRegistry,
        attribution: AttributionRegistry,
    ) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            state,
            ingest,
            attribution,
            events,
            in_flight: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WatcherEvent> {
        self.events.subscribe()
    }

    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }

    pub fn ingest_registry(&self) -> &IngestRegistry {
        &self.ingest
    }

    pub fn attribution_registry(&self) -> &AttributionRegistry {
        &self.attribution
    }

    pub async fn full_rescan(&self) -> u64 {
        scan::full_rescan(self).await
    }

    pub async fn run(self) -> WatcherHandle {
        ingest_loop::run(Arc::new(self)).await
    }
}
