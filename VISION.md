# Vision

> Make AI usage at work as legible as time, with zero friction for
> the developer doing the work.

## The problem

AI usage is invisible. A developer might burn $200 of opus tokens on a
single Friday afternoon and nobody — not them, not their lead, not
finance — knows until the monthly bill arrives. Existing tools either
require behaviour change (run my CLI wrapper, switch to my proxy) or
report at the org level so the individual loop never closes.

Pulse closes that loop. It reads what's already on disk, attributes
it to the right ticket, and surfaces it where the developer already
looks (the menu bar). No interception, no behaviour change, no telemetry.

## Principles

1. **Local-first.** All data lives in SQLite under your home directory.
   Nothing is sent anywhere unless you explicitly enable a Jira/Linear/
   etc. integration. The watcher is read-only on `~/.claude/projects`.

2. **Zero behaviour change.** If you have to remember to do something,
   the product has failed. Pulse uses the transcripts Claude Code
   already writes — no wrappers, no `pulse run`, no environment
   variables.

3. **One concern per file.** New providers (attribution sources,
   ingest sources, exporters) are new files in `extensions/`. The
   trait surface is small enough that a Linear plugin should fit on
   one screen.

4. **Plugin-shaped, not framework-shaped.** Pulse's job is to be the
   loop — watcher → ingest → attribution → storage → UI. Everything
   else is a plugin: providers, integrations, exporters. The loop
   itself is finished and small.

5. **Native, not Electron.** Tauri's WKWebView/WebView2/WebKitGTK is
   ~5MB resident; Electron is ~150MB. For a tray-resident app that
   runs all day, native footprints matter.

## Three-year sketch

**Year one — individual.** Pulse on your machine, attributing your
turns, showing your meters. Claude Code today; Codex CLI and Gemini
CLI as drop-in `IngestProvider` extensions.

**Year two — team.** A sync service (opt-in) that aggregates per-task
totals from individual Pulse installs into a shared dashboard. Per-team
budgets, alerts when a sprint's AI spend hits 80% of plan, weekly
digests. Still local-first per developer; the sync service only sees
attributed totals, never transcripts.

**Year three — the loop closes.** Pulse becomes the link between
"what we're spending" and "what we're shipping." Per-ticket cost flows
into Jira/Linear as a custom field. Engineering leads see "PROJ-123
shipped in 3 days, used $42 of AI" as a routine data point in retros.

## Non-goals

- **Not a code-quality tool.** Pulse measures AI usage, not code
  quality. We don't try to tell you whether the AI was helpful.
- **Not a replacement for the provider's own dashboards.** Anthropic
  and OpenAI both ship console dashboards that show org-level totals.
  Pulse is for the per-developer, per-task layer those tools don't see.
- **Not a billing system.** Pricing displayed in Pulse is approximate
  (matches each provider's published list). For invoiced totals,
  trust the provider.

## What we won't compromise on

- **No telemetry.** Ever. We don't even count installs.
- **No cloud-only mode.** The desktop app must work offline forever
  with the local data it already has.
- **No closed plugin API.** If a plugin can ship in Pulse's tree, it
  can ship out of tree as an `extensions/<x>` Cargo crate.
