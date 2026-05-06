use std::sync::Arc;

use chrono::{Duration, Utc};
use pulse_core::{ActiveTask, DashboardSummary, DateRange, OnboardingStatus, Settings};
use tauri::{AppHandle, Manager, State};

use crate::state::ShellState;

type Arg<'a> = State<'a, Arc<ShellState>>;

#[tauri::command]
pub async fn get_active_task(state: Arg<'_>) -> Result<ActiveTask, String> {
    state.state.snapshot().map_err(err)
}

#[tauri::command]
pub async fn get_dashboard(state: Arg<'_>, days: Option<u32>) -> Result<DashboardSummary, String> {
    let to = Utc::now();
    let from = to - Duration::days(days.unwrap_or(7).max(1) as i64);
    let db = state.state.db();
    let totals = db.usage_totals(from, to).map_err(err)?;
    let mut tasks = db.usage_by_task(from, to).map_err(err)?;
    let unattributed = tasks
        .iter()
        .find(|t| t.task_id.is_none())
        .map(|t| t.usage.clone())
        .unwrap_or_default();
    tasks.retain(|t| t.task_id.is_some());
    let daily = db.daily_series(from, to).map_err(err)?;
    let model_share = db.model_share(from, to).map_err(err)?;
    Ok(DashboardSummary {
        range: DateRange { from, to },
        totals,
        tasks,
        unattributed,
        daily,
        model_share,
    })
}

#[tauri::command]
pub async fn get_settings(state: Arg<'_>) -> Result<Settings, String> {
    Ok(state.state.settings())
}

#[tauri::command]
pub async fn save_settings(state: Arg<'_>, settings: Settings) -> Result<(), String> {
    state.state.save_settings(settings).map_err(err)
}

#[tauri::command]
pub async fn get_onboarding_status(state: Arg<'_>) -> Result<OnboardingStatus, String> {
    let root = state.primary_watch_root();
    let found = root.as_ref().map(|p| p.exists()).unwrap_or(false);
    Ok(OnboardingStatus {
        claude_dir_found: found && state.claude_code_present(),
        claude_dir_path: root.map(|p| p.to_string_lossy().to_string()),
        sessions_discovered: state.discovered_sessions(),
        ingest_complete: !state.should_show_onboarding(),
    })
}

#[tauri::command]
pub async fn open_dashboard(handle: AppHandle) -> Result<(), String> {
    if let Some(win) = handle.get_webview_window("dashboard") {
        win.show().map_err(err)?;
        win.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_settings(handle: AppHandle) -> Result<(), String> {
    if let Some(win) = handle.get_webview_window("settings") {
        win.show().map_err(err)?;
        win.set_focus().map_err(err)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reset_database(state: Arg<'_>) -> Result<(), String> {
    state.state.db().reset().map_err(err)
}

#[tauri::command]
pub async fn trigger_full_rescan(state: Arg<'_>) -> Result<u64, String> {
    Ok(state.watcher.full_rescan().await)
}

fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e}")
}
