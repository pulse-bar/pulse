use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use parking_lot::RwLock;
use pulse_attribution::Registry as AttributionRegistry;
use pulse_core::{AppState, Db};
use pulse_ingest::Registry as IngestRegistry;
use pulse_watcher::{Watcher, WatcherHandle};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const ONBOARDING_KEY: &str = "pulse.onboarded";

pub struct ShellState {
    pub state: Arc<AppState>,
    pub watcher: Arc<Watcher>,
    #[allow(dead_code)]
    handle: Mutex<Option<WatcherHandle>>,
    onboarded_seen: RwLock<bool>,
    ingest: IngestRegistry,
}

impl ShellState {
    pub async fn initialize(app: AppHandle) -> anyhow::Result<Self> {
        let data_dir = data_dir(&app)?;
        let db = Db::open(&data_dir.join("pulse.db")).context("open db")?;
        let onboarded = db
            .get_setting(ONBOARDING_KEY)
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(false);

        let app_state = Arc::new(AppState::new(db).context("init app state")?);

        let ingest = IngestRegistry::with_defaults();
        let attribution = AttributionRegistry::with_defaults();
        let watcher = Arc::new(Watcher::new(app_state.clone(), ingest.clone(), attribution));

        let handle = (*watcher).clone().run().await;

        Ok(Self {
            state: app_state,
            watcher,
            handle: Mutex::new(Some(handle)),
            onboarded_seen: RwLock::new(onboarded),
            ingest,
        })
    }

    pub fn should_show_onboarding(&self) -> bool {
        !*self.onboarded_seen.read()
    }

    pub fn mark_onboarded(&self) {
        *self.onboarded_seen.write() = true;
        let _ = self.state.db().set_setting(ONBOARDING_KEY, "true");
    }

    pub fn primary_watch_root(&self) -> Option<PathBuf> {
        self.ingest
            .providers()
            .iter()
            .flat_map(|p| p.watch_roots())
            .next()
    }

    pub fn discovered_sessions(&self) -> u64 {
        self.state.db().count_sessions().unwrap_or(0)
    }

    pub fn claude_code_present(&self) -> bool {
        self.ingest
            .providers()
            .iter()
            .any(|p| p.name() == "claude-code")
    }

    #[allow(dead_code)]
    pub async fn shutdown(self: Arc<Self>) {
        if let Some(handle) = self.handle.lock().await.take() {
            handle.shutdown().await;
        }
    }
}

fn data_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
    let dir = app.path().app_data_dir().context("app_data_dir")?;
    std::fs::create_dir_all(&dir).ok();
    Ok(dir)
}
