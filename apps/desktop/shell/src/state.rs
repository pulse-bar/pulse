use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use parking_lot::RwLock;
use pulse_attribution::Registry as AttributionRegistry;
use pulse_auth::{CredentialStore, KeychainCredentialStore, OAuthClient};
use pulse_core::{AppState, Db};
use pulse_enrichment::{EnrichmentDaemon, EnrichmentHandle, Registry as EnrichmentRegistry};
use pulse_ext_jira::{JiraEnricher, JiraPlugin};
use pulse_plugins::PluginRegistry;
use pulse_ingest::Registry as IngestRegistry;
use pulse_watcher::{Watcher, WatcherHandle};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const ONBOARDING_KEY: &str = "pulse.onboarded";

pub struct ShellState {
    pub state: Arc<AppState>,
    pub watcher: Arc<Watcher>,
    pub enrichment: EnrichmentDaemon,
    pub credentials: Arc<dyn CredentialStore>,
    pub oauth: OAuthClient,
    pub plugins: PluginRegistry,
    #[allow(dead_code)]
    watcher_handle: Mutex<Option<WatcherHandle>>,
    #[allow(dead_code)]
    enrichment_handle: Mutex<Option<EnrichmentHandle>>,
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
        let watcher = Arc::new(Watcher::new(
            app_state.clone(),
            ingest.clone(),
            attribution,
        ));
        let watcher_handle = (*watcher).clone().run().await;

        let credentials: Arc<dyn CredentialStore> = Arc::new(KeychainCredentialStore::new());

        let enrichment_registry = EnrichmentRegistry::new();
        enrichment_registry.register(Arc::new(JiraEnricher::new(credentials.clone())));
        let enrichment = EnrichmentDaemon::new(app_state.clone(), enrichment_registry);
        let enrichment_handle = enrichment.clone().run().await;

        let plugins = PluginRegistry::new();
        plugins.register(Arc::new(JiraPlugin::new(credentials.clone())));

        Ok(Self {
            state: app_state,
            watcher,
            enrichment,
            credentials,
            oauth: OAuthClient::new(),
            plugins,
            watcher_handle: Mutex::new(Some(watcher_handle)),
            enrichment_handle: Mutex::new(Some(enrichment_handle)),
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
        if let Some(h) = self.watcher_handle.lock().await.take() {
            h.shutdown().await;
        }
        if let Some(h) = self.enrichment_handle.lock().await.take() {
            h.shutdown().await;
        }
    }
}

fn data_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
    let dir = app.path().app_data_dir().context("app_data_dir")?;
    std::fs::create_dir_all(&dir).ok();
    Ok(dir)
}
