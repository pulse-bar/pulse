mod bridge;
mod commands;
mod state;
mod tray;

use std::sync::Arc;

use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::state::ShellState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(win) = app.get_webview_window("dashboard") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_active_task,
            commands::get_dashboard,
            commands::get_settings,
            commands::save_settings,
            commands::get_onboarding_status,
            commands::open_dashboard,
            commands::open_settings,
            commands::reset_database,
            commands::trigger_full_rescan,
            commands::get_task_metadata,
            commands::get_enrichment_status,
            commands::run_enrichment_now,
            commands::save_jira_sites,
            commands::upsert_jira_site,
            commands::delete_jira_site,
            commands::store_jira_token,
            commands::jira_token_present,
            commands::delete_jira_token,
            commands::test_jira_site,
            commands::connect_jira_oauth,
            commands::oauth_begin,
            commands::oauth_complete,
            commands::list_plugins,
            commands::list_plugin_statuses,
            commands::list_plugin_instances,
            commands::test_plugin_instance,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let shell = tauri::async_runtime::block_on(ShellState::initialize(handle.clone()))
                .expect("initialise shell state");
            let shell = Arc::new(shell);
            app.manage(shell.clone());

            tray::build(&handle)?;
            install_close_to_tray(&handle);

            let bridge_handle = handle.clone();
            let bridge_state = shell.clone();
            tauri::async_runtime::spawn(async move {
                bridge::pump(bridge_handle, bridge_state).await;
            });

            let enrichment_handle = handle.clone();
            let enrichment_state = shell.clone();
            tauri::async_runtime::spawn(async move {
                bridge::pump_enrichment(enrichment_handle, enrichment_state).await;
            });

            // 1Hz tick keeps meters animating during idle ticks.
            let tick_handle = handle.clone();
            let tick_state = shell.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    bridge::emit_active_task(&tick_handle, &tick_state);
                }
            });

            let initial_window = if shell.should_show_onboarding() {
                "onboarding"
            } else {
                "dashboard"
            };
            if let Some(win) = handle.get_webview_window(initial_window) {
                let _ = win.show();
                let _ = win.set_focus();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("run pulse desktop");
}

fn install_close_to_tray(handle: &tauri::AppHandle) {
    for label in ["popover", "dashboard", "settings", "onboarding"] {
        if let Some(win) = handle.get_webview_window(label) {
            let h = handle.clone();
            let l = label.to_string();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(w) = h.get_webview_window(&l) {
                        let _ = w.hide();
                    }
                }
            });
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,pulse_desktop=debug,pulse_watcher=debug"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().compact())
        .try_init();
}
