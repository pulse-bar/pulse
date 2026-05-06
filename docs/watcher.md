# Watcher

> How `crates/watcher` works. Read this before touching file-watching,
> ingestion concurrency, or rotation.

## Lifecycle

```rust
let watcher = Watcher::new(state, ingest_registry, attribution_registry);
let mut events = watcher.subscribe();
let handle = watcher.run().await;       // initial scan + live watch
// ...
handle.shutdown().await;
```

`run()` does two things in sequence:

1. **Initial scan** — synchronous; walks `IngestProvider::watch_roots`,
   ingests matching files, persists offsets.
2. **Live loop** — async; spawns the FS-watcher thread + tokio event
   loop. Returns a `WatcherHandle` whose `shutdown()` is the
   deterministic stop.

## Two layers (notify ↔ tokio)

`notify`'s callback is synchronous. We must not block on SQLite there.
The bridge is:

```
notify thread  ──blocking_send──▶  tokio mpsc  ──recv──▶  async fanout
```

Each fanout tick:
1. Receives one path.
2. Drains anything else queued (`while let Ok(extra) = rx.try_recv()`).
3. Dedupes into a `HashSet`.
4. Spawns one `tokio::spawn` per path.
5. Awaits all before returning to `select!`.

Result: bursts batch cleanly; the same file written three times runs
once; different files run in parallel.

## Per-path serialisation

If notify emits two events for the same file back-to-back (e.g.
`Modify(Data)` + `Modify(Metadata)`) we still want exactly one ingest
pass at a time so byte offsets stay consistent. The watcher holds:

```rust
HashMap<PathBuf, Arc<tokio::Mutex<()>>>
```

and acquires the per-path mutex before reading. Different paths run in
parallel; the same path serialises.

## Byte-offset bookkeeping

Every JSONL has a `sessions.file_offset` row. On each ingest:

1. Read offset from SQLite.
2. Stat the file.
3. If `file_size < offset` → rotation/truncation; reset to 0 and emit
   `WatcherEvent::FileRotated`.
4. Seek to offset; tail to EOF, parsing each line.
5. After EOF, persist `bytes_consumed` back to SQLite atomically with
   the session row.

A crash mid-tail loses at most the last partial buffer; the next start
re-reads from the persisted offset. Since `message_id` is the dedup PK,
re-ingesting an already-stored turn is idempotent.

## Backpressure

- The mpsc channel is bounded (512). When notify produces faster than
  we can ingest, `blocking_send` blocks the notify thread, applying
  pressure all the way back to the OS event source.
- The broadcast channel for `WatcherEvent` is bounded (256). Lagging
  subscribers see `RecvError::Lagged(n)` and decide to recover or
  restart.

## Failure isolation

A bad line in one transcript doesn't stop ingestion of others:

- `IngestProvider::parse_line` returning `Err` → trace, skip line.
- `Db::upsert_turn` returning `Err` → emit `WatcherEvent::Error`,
  continue.
- `notify::watch` failing on a missing root → log, skip that root.
  Pulse won't crash if `~/.claude/projects` doesn't exist yet.

## Testing

The `WatcherEvent` broadcast is the public observation point:

```rust
let watcher = Watcher::new(state, ingest, attribution);
let mut events = watcher.subscribe();
let handle = watcher.run().await;

std::fs::OpenOptions::new()
    .append(true)
    .open("tests/fixtures/sample.jsonl")?
    .write_all(b"{...}\n")?;

let event = tokio::time::timeout(Duration::from_secs(2), events.recv()).await??;
assert!(matches!(event, WatcherEvent::TurnIngested { .. }));

handle.shutdown().await;
```

## Performance notes

- SQLite WAL + `synchronous = NORMAL`. Tens of writes/sec sustain
  comfortably; bottleneck is `notify`'s 50ms debounce floor.
- `r2d2` keeps a pool of 8 connections. Per-upsert borrows are short.
- Rolling-window cache hit rate is recomputed per turn, not per
  snapshot — keeps the read path cheap.
