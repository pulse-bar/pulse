use chrono::Utc;
use parking_lot::RwLock;

use crate::error::PulseResult;
use crate::model::{
    ActiveTask, AttributionConfidence, AttributionOutcome, SessionState, Settings, TaskSnapshot,
    UsageTotals,
};
use crate::storage::Db;
use crate::time::{session_window, weekly_reset_after, IDLE_AFTER_SECONDS};
use crate::turn::ParsedTurn;

#[derive(Default, Clone)]
struct LiveWindow {
    started_at: Option<chrono::DateTime<Utc>>,
    last_activity: Option<chrono::DateTime<Utc>>,
    active_task_id: Option<String>,
    active_branch: Option<String>,
    active_cwd: Option<String>,
    active_model: Option<String>,
    active_confidence: Option<AttributionConfidence>,
    active_confidence_score: f64,
    usage: UsageTotals,
    last_state: Option<SessionState>,
}

pub struct AppState {
    db: Db,
    settings: RwLock<Settings>,
    window: RwLock<LiveWindow>,
}

impl AppState {
    pub fn new(db: Db) -> PulseResult<Self> {
        let settings = db.load_settings().unwrap_or_default();
        Ok(Self {
            db,
            settings: RwLock::new(settings),
            window: RwLock::new(LiveWindow::default()),
        })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().clone()
    }

    pub fn save_settings(&self, s: Settings) -> PulseResult<()> {
        self.db.save_settings(&s)?;
        *self.settings.write() = s;
        Ok(())
    }

    pub fn observe(&self, turn: &ParsedTurn, outcome: &AttributionOutcome, cost_usd: f64) {
        let mut win = self.window.write();
        let now = turn.ts;
        let alive = matches!(
            (win.started_at, win.last_activity),
            (Some(_), Some(last)) if now.signed_duration_since(last) <= session_window()
        );

        if !alive {
            win.started_at = Some(now);
            win.usage = UsageTotals::default();
        }

        win.last_activity = Some(now);
        win.usage.input_tokens += turn.input_tokens;
        win.usage.output_tokens += turn.output_tokens;
        win.usage.cache_creation_tokens += turn.cache_creation_tokens;
        win.usage.cache_read_tokens += turn.cache_read_tokens;
        win.usage.total_tokens += turn.total_tokens();
        win.usage.cost_usd += cost_usd;
        win.usage.calls += 1;
        win.usage.recompute_cache_hit_rate();

        win.active_task_id = outcome.task_id.clone().or(win.active_task_id.clone());
        win.active_branch = turn.branch.clone().or(win.active_branch.clone());
        win.active_cwd = turn.cwd.clone().or(win.active_cwd.clone());
        win.active_model = turn.model.clone().or(win.active_model.clone());
        win.active_confidence = Some(outcome.confidence);
        win.active_confidence_score = outcome.score;
    }

    pub fn snapshot(&self) -> PulseResult<ActiveTask> {
        let settings = self.settings();
        let win = self.window.read().clone();
        let now = Utc::now();

        let session_used_pct = if settings.session_token_budget == 0 {
            0.0
        } else {
            win.usage.total_tokens as f64 / settings.session_token_budget as f64
        };
        let session_reset_at = win.started_at.map(|s| s + session_window());

        let weekly_start = crate::time::monday_start(now);
        let weekly_totals = self.db.usage_totals(weekly_start, now).unwrap_or_default();
        let weekly_used_pct = if settings.weekly_token_budget == 0 {
            0.0
        } else {
            weekly_totals.total_tokens as f64 / settings.weekly_token_budget as f64
        };

        let idle = win
            .last_activity
            .map(|t| now.signed_duration_since(t).num_seconds() > IDLE_AFTER_SECONDS)
            .unwrap_or(true);

        let state = if idle {
            SessionState::Idle
        } else if session_used_pct >= settings.crit_threshold_pct
            || weekly_used_pct >= settings.crit_threshold_pct
        {
            SessionState::Crit
        } else if session_used_pct >= settings.warn_threshold_pct
            || weekly_used_pct >= settings.warn_threshold_pct
        {
            SessionState::Warn
        } else {
            SessionState::Normal
        };

        let task = win.active_task_id.as_deref().map(|tid| TaskSnapshot {
            task_id: Some(tid.into()),
            task_name: Some(tid.into()),
            branch: win.active_branch.clone(),
            cwd: win.active_cwd.clone(),
            model: win.active_model.clone(),
            confidence: win.active_confidence.unwrap_or(AttributionConfidence::Low),
            confidence_score: win.active_confidence_score,
            usage: win.usage.clone(),
            first_seen: win.started_at.unwrap_or(now),
            last_seen: win.last_activity.unwrap_or(now),
        });

        self.window.write().last_state = Some(state);

        Ok(ActiveTask {
            task,
            session_used_pct,
            session_reset_at,
            weekly_used_pct,
            weekly_reset_at: Some(weekly_reset_after(now)),
            state,
        })
    }
}
