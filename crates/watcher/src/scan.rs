use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pulse_core::{pricing, ParsedTurn};
use pulse_ingest::IngestProvider;

use crate::{Watcher, WatcherEvent};

pub async fn ingest_path(watcher: &Watcher, path: &Path) -> std::io::Result<u64> {
    let Some(provider) = watcher.ingest_registry().provider_for(path) else {
        return Ok(0);
    };

    // Per-path mutex prevents two concurrent reads racing on the same offset.
    let lock = {
        let mut map = watcher.in_flight.lock();
        map.entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    };
    let _guard = lock.lock().await;

    ingest_path_inner(watcher, provider.as_ref(), path)
}

fn ingest_path_inner(
    watcher: &Watcher,
    provider: &dyn IngestProvider,
    path: &Path,
) -> std::io::Result<u64> {
    let file_path_str = path.to_string_lossy().to_string();
    let db = watcher.state().db();

    let start_offset = db.session_offset(&file_path_str).unwrap_or(0);

    let file = std::fs::File::open(path)?;
    let total_size = file.metadata()?.len();

    // Truncation/rotation: re-read from byte zero so we don't silently skip turns.
    let resume_offset = if total_size < start_offset {
        let _ = watcher.events.send(WatcherEvent::FileRotated {
            path: path.to_path_buf(),
        });
        0
    } else {
        start_offset
    };

    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(resume_offset))?;

    let project = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string());

    let mut bytes_consumed = resume_offset;
    let mut count: u64 = 0;
    let mut latest: Option<ParsedTurn> = None;

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        bytes_consumed += n as u64;

        let parsed = match provider.parse_line(&line) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let Some(turn) = parsed else { continue };

        let settings = watcher.state().settings();
        let outcome = watcher.attribution_registry().resolve(&turn, &settings);
        let cost = pricing::cost_of(&turn);

        if let Err(err) = db.upsert_turn(
            &turn,
            outcome.task_id.as_deref(),
            outcome.confidence,
            outcome.score,
            cost,
        ) {
            let _ = watcher.events.send(WatcherEvent::Error {
                path: Some(path.to_path_buf()),
                message: format!("{err}"),
            });
            continue;
        }

        watcher.state().observe(&turn, &outcome, cost);
        let _ = watcher.events.send(WatcherEvent::TurnIngested {
            turn: turn.clone(),
            outcome: outcome.clone(),
            cost_usd: cost,
        });

        latest = Some(turn);
        count += 1;
    }

    if let Some(turn) = latest {
        let _ = db.upsert_session(
            &turn.session_id,
            turn.cwd.as_deref(),
            turn.branch.as_deref(),
            turn.model.as_deref(),
            project.as_deref(),
            turn.provider,
            &file_path_str,
            turn.ts,
        );
        let _ = db.set_session_offset(&file_path_str, bytes_consumed);
    }

    Ok(count)
}

pub fn discover_all(watcher: &Watcher) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for provider in watcher.ingest_registry().providers() {
        for root in provider.watch_roots() {
            walk(provider.as_ref(), &root, &mut out);
        }
    }
    out
}

fn walk(provider: &dyn IngestProvider, root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(provider, &p, out);
        } else if provider.matches(&p) {
            out.push(p);
        }
    }
}

pub async fn full_rescan(watcher: &Watcher) -> u64 {
    let files = discover_all(watcher);
    let mut total = 0u64;
    for path in files {
        if watcher
            .state()
            .db()
            .set_session_offset(&path.to_string_lossy(), 0)
            .is_err()
        {
            continue;
        }
        if let Ok(n) = ingest_path(watcher, &path).await {
            total += n;
        }
    }
    total
}
