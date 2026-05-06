use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::scan;
use crate::{Watcher, WatcherEvent};

pub struct WatcherHandle {
    shutdown: Option<oneshot::Sender<()>>,
    join: JoinHandle<()>,
}

impl WatcherHandle {
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = self.join.await;
    }
}

pub async fn run(watcher: Arc<Watcher>) -> WatcherHandle {
    initial_scan(&watcher).await;

    let (path_tx, mut path_rx) = mpsc::channel::<PathBuf>(512);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let debouncer = build_debouncer(&watcher, path_tx);

    let watcher_clone = watcher.clone();
    let join = tokio::spawn(async move {
        // notify's debouncer thread halts the moment this is dropped.
        let _debouncer = debouncer;

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                Some(path) = path_rx.recv() => {
                    let mut batch: HashSet<PathBuf> = HashSet::new();
                    batch.insert(path);
                    while let Ok(extra) = path_rx.try_recv() {
                        batch.insert(extra);
                    }

                    let mut tasks = Vec::with_capacity(batch.len());
                    for p in batch {
                        let w = watcher_clone.clone();
                        tasks.push(tokio::spawn(async move {
                            if let Err(err) = scan::ingest_path(&w, &p).await {
                                tracing::warn!("ingest {p:?} failed: {err}");
                            }
                        }));
                    }
                    for t in tasks {
                        let _ = t.await;
                    }
                }
                else => break,
            }
        }
    });

    WatcherHandle {
        shutdown: Some(shutdown_tx),
        join,
    }
}

async fn initial_scan(watcher: &Arc<Watcher>) {
    let files = scan::discover_all(watcher);
    let _ = watcher
        .events
        .send(WatcherEvent::InitialScanStarted {
            roots: collected_roots(watcher),
        });

    let total = files.len();
    let mut processed = 0;
    for path in &files {
        if let Err(err) = scan::ingest_path(watcher, path).await {
            let _ = watcher.events.send(WatcherEvent::Error {
                path: Some(path.clone()),
                message: format!("{err}"),
            });
        }
        processed += 1;
        let _ = watcher.events.send(WatcherEvent::InitialScanProgress {
            discovered: total,
            processed,
        });
    }
    let _ = watcher.events.send(WatcherEvent::InitialScanComplete {
        total_turns: watcher.state().db().count_sessions().unwrap_or(0),
    });
}

fn build_debouncer(
    watcher: &Watcher,
    path_tx: mpsc::Sender<PathBuf>,
) -> Debouncer<RecommendedWatcher, FileIdMap> {
    let debounce_ms = watcher.state().settings().poll_interval_ms.max(50);
    let providers = watcher.ingest_registry().providers();

    let mut debouncer = new_debouncer(
        Duration::from_millis(debounce_ms),
        None,
        move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            for event in events {
                for path in &event.paths {
                    if providers.iter().any(|p| p.matches(path)) {
                        let _ = path_tx.blocking_send(path.clone());
                    }
                }
            }
        },
    )
    .expect("build debouncer");

    for root in collected_roots(watcher) {
        if !root.exists() {
            continue;
        }
        if let Err(err) = debouncer.watcher().watch(&root, RecursiveMode::Recursive) {
            tracing::warn!("watch {} failed: {err}", root.display());
        } else {
            tracing::info!("watching {}", root.display());
        }
    }

    debouncer
}

fn collected_roots(watcher: &Watcher) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for provider in watcher.ingest_registry().providers() {
        for root in provider.watch_roots() {
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
    }
    roots
}
