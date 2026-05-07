// Translates pulse-watcher's broadcast events into Tauri IPC events.

use std::sync::Arc;

use pulse_core::SessionState;
use pulse_enrichment::EnrichmentEvent;
use pulse_watcher::WatcherEvent;
use tauri::{AppHandle, Emitter};

use crate::state::ShellState;

pub async fn pump_enrichment(handle: AppHandle, state: Arc<ShellState>) {
    let mut rx = state.enrichment.subscribe();
    loop {
        match rx.recv().await {
            Ok(EnrichmentEvent::TaskEnriched { metadata }) => {
                let _ = handle.emit("pulse://task-enriched", &metadata);
            }
            Ok(EnrichmentEvent::StatusChanged { status }) => {
                let _ = handle.emit("pulse://enrichment-status", &status);
            }
            Ok(EnrichmentEvent::Error { task_id, message }) => {
                let _ = handle.emit(
                    "pulse://enrichment-error",
                    serde_json::json!({ "taskId": task_id, "message": message }),
                );
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

pub async fn pump(handle: AppHandle, state: Arc<ShellState>) {
    let mut rx = state.watcher.subscribe();
    let mut last_state: Option<SessionState> = None;

    loop {
        match rx.recv().await {
            Ok(WatcherEvent::InitialScanStarted { roots }) => {
                let _ = handle.emit(
                    "pulse://ingest-progress",
                    serde_json::json!({
                        "phase": "initial-scan-started",
                        "roots": roots.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    }),
                );
            }
            Ok(WatcherEvent::InitialScanProgress { discovered, processed }) => {
                let _ = handle.emit(
                    "pulse://ingest-progress",
                    serde_json::json!({
                        "phase": "initial-scan-progress",
                        "discovered": discovered,
                        "processed": processed,
                    }),
                );
            }
            Ok(WatcherEvent::InitialScanComplete { total_turns }) => {
                state.mark_onboarded();
                let _ = handle.emit(
                    "pulse://ingest-progress",
                    serde_json::json!({
                        "phase": "initial-scan-complete",
                        "totalTurns": total_turns,
                    }),
                );
                emit_active_task(&handle, &state);
            }
            Ok(WatcherEvent::TurnIngested { turn, .. }) => {
                let _ = handle.emit(
                    "pulse://usage-updated",
                    serde_json::json!({
                        "messageId": turn.message_id,
                        "sessionId": turn.session_id,
                    }),
                );
                if let Some(s) = emit_active_task(&handle, &state) {
                    if last_state != Some(s) {
                        last_state = Some(s);
                        if matches!(s, SessionState::Warn | SessionState::Crit) {
                            let _ = handle.emit(
                                "pulse://threshold-crossed",
                                serde_json::json!({ "state": s }),
                            );
                        }
                    }
                }
            }
            Ok(WatcherEvent::FileRotated { path }) => {
                tracing::info!("rotated: {}", path.display());
            }
            Ok(WatcherEvent::Error { path, message }) => {
                let _ = handle.emit(
                    "pulse://error",
                    serde_json::json!({
                        "path": path.map(|p| p.to_string_lossy().to_string()),
                        "message": message,
                    }),
                );
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("event bus lagged by {n}");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

pub fn emit_active_task(handle: &AppHandle, state: &Arc<ShellState>) -> Option<SessionState> {
    match state.state.snapshot() {
        Ok(active) => {
            let s = active.state;
            let _ = handle.emit("pulse://active-task-changed", &active);
            Some(s)
        }
        Err(err) => {
            tracing::warn!("snapshot failed: {err}");
            None
        }
    }
}
