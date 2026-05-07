// Wire format mirrors `packages/types/src/types.ts`. Both sides use camelCase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttributionConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Normal,
    Warn,
    Crit,
    Idle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub calls: u64,
    pub cache_hit_rate: f64,
}

impl UsageTotals {
    pub fn recompute_cache_hit_rate(&mut self) {
        let touches = self.cache_creation_tokens + self.cache_read_tokens;
        self.cache_hit_rate = if touches == 0 {
            0.0
        } else {
            self.cache_read_tokens as f64 / touches as f64
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    pub task_id: Option<String>,
    pub task_name: Option<String>,
    pub branch: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub confidence: AttributionConfidence,
    pub confidence_score: f64,
    pub usage: UsageTotals,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub metadata: Option<TaskMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTask {
    pub task: Option<TaskSnapshot>,
    pub session_used_pct: f64,
    pub session_reset_at: Option<DateTime<Utc>>,
    pub weekly_used_pct: f64,
    pub weekly_reset_at: Option<DateTime<Utc>>,
    pub state: SessionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPoint {
    pub date: String,
    pub tokens: u64,
    pub cost: f64,
    pub calls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelShare {
    pub model: String,
    pub tokens: u64,
    pub pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub range: DateRange,
    pub totals: UsageTotals,
    pub tasks: Vec<TaskSnapshot>,
    pub unattributed: UsageTotals,
    pub daily: Vec<DailyPoint>,
    pub model_share: Vec<ModelShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub branch_regex: String,
    pub poll_interval_ms: u64,
    pub weekly_token_budget: u64,
    pub session_token_budget: u64,
    pub warn_threshold_pct: f64,
    pub crit_threshold_pct: f64,
    pub notify_on_warn: bool,
    pub notify_on_crit: bool,
    pub notify_daily_summary: bool,
    pub appearance: String,
    pub start_at_login: bool,

    pub enrichment_enabled: bool,
    pub enrichment_interval_secs: u64,
    pub enrichment_cache_ttl_secs: u64,

    pub jira: JiraConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            branch_regex: r"(?i)([A-Z][A-Z0-9]+-\d+)".into(),
            poll_interval_ms: 250,
            weekly_token_budget: 5_000_000,
            session_token_budget: 200_000,
            warn_threshold_pct: 0.78,
            crit_threshold_pct: 0.92,
            notify_on_warn: true,
            notify_on_crit: true,
            notify_daily_summary: true,
            appearance: "dark".into(),
            start_at_login: true,

            enrichment_enabled: true,
            enrichment_interval_secs: 30,
            enrichment_cache_ttl_secs: 6 * 60 * 60,

            jira: JiraConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraConfig {
    pub sites: Vec<JiraSite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JiraSite {
    pub id: String,
    pub label: String,
    pub base_url: String,
    pub project_keys: Vec<String>,
    pub auth_kind: JiraAuthKind,
    pub email: Option<String>,
    pub oauth_client_id: Option<String>,
    pub enabled: bool,
}

impl Default for JiraSite {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            base_url: String::new(),
            project_keys: vec![],
            auth_kind: JiraAuthKind::OAuth2,
            email: None,
            oauth_client_id: None,
            enabled: true,
        }
    }
}

impl Default for JiraAuthKind {
    fn default() -> Self {
        JiraAuthKind::OAuth2
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JiraAuthKind {
    None,
    Bearer,
    Basic,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMetadata {
    pub task_id: String,
    pub enricher: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub url: Option<String>,
    pub project_key: Option<String>,
    pub issue_type: Option<String>,
    pub priority: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EnrichmentState {
    Idle,
    Running,
    Disabled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentStatus {
    pub state: EnrichmentState,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub pending_count: u64,
    pub enrichers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStatus {
    pub claude_dir_found: bool,
    pub claude_dir_path: Option<String>,
    pub sessions_discovered: u64,
    pub ingest_complete: bool,
}

#[derive(Debug, Clone)]
pub struct AttributionOutcome {
    pub task_id: Option<String>,
    pub confidence: AttributionConfidence,
    pub score: f64,
}

impl AttributionOutcome {
    pub fn unattributed() -> Self {
        Self {
            task_id: None,
            confidence: AttributionConfidence::Low,
            score: 0.0,
        }
    }
}
