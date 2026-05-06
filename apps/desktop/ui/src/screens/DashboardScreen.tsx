import { useState } from 'react';
import { formatCostUsd, formatPct, formatTokens } from '@pulse/types';
import type { TaskSnapshot } from '@pulse/types';
import { useDashboard } from '../hooks/useDashboard';
import { useActiveTask } from '../hooks/useActiveTask';
import { PulseLogoMark } from '../components/PulseLogo';
import { ToastHost } from '../components/ToastHost';
import { openSettingsWindow } from '../lib/tauri';

type Tab = 'tasks' | 'timeline' | 'models' | 'team';

const RANGE_OPTIONS = [
  { label: 'Today', days: 1 },
  { label: '7d', days: 7 },
  { label: '30d', days: 30 },
  { label: '90d', days: 90 },
];

export function DashboardScreen() {
  const [days, setDays] = useState(7);
  const [tab, setTab] = useState<Tab>('tasks');
  const { data, loading } = useDashboard(days);
  const active = useActiveTask();

  return (
    <div className="window">
      <div className="dash">
        <header className="dash-header">
          <div className="dash-logo">
            <PulseLogoMark size={22} />
            <div>
              <div className="dash-logo-name">Pulse</div>
              <div className="dash-logo-version">v0.1.0 · {days}d window</div>
            </div>
          </div>

          {active.task && (
            <>
              <span className="dash-vdiv" />
              <div className="active-chip">
                <span className="active-chip-dot" />
                <span className="active-chip-id">{active.task.taskId}</span>
                <span className="active-chip-name">{active.task.branch ?? ''}</span>
              </div>
            </>
          )}

          <div className="dash-meters">
            <HeaderMeter label="Session" pct={active.sessionUsedPct} value={formatPct(active.sessionUsedPct)} tone="s" />
            <HeaderMeter label="Weekly" pct={active.weeklyUsedPct} value={formatPct(active.weeklyUsedPct)} tone="w" />
          </div>

          <div className="dash-qstats">
            <QStat value={formatTokens(data?.totals.totalTokens ?? 0)} label="Tokens" tone="pulse" />
            <QStat value={formatCostUsd(data?.totals.costUsd ?? 0)} label="Cost" tone="green" />
            <QStat value={String(data?.totals.calls ?? 0)} label="Calls" tone="cyan" />
          </div>
        </header>

        <div className="dash-tabs">
          {(['tasks', 'timeline', 'models', 'team'] as Tab[]).map((t) => (
            <button key={t} className={`dash-tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
              {labelFor(t)}
            </button>
          ))}
          <div className="dash-tab-right">
            <button className="dash-tab" onClick={() => openSettingsWindow()}>Settings</button>
          </div>
        </div>

        <div className="date-chips">
          {RANGE_OPTIONS.map((r) => (
            <button key={r.label} className={`date-chip ${days === r.days ? 'active' : ''}`} onClick={() => setDays(r.days)}>
              {r.label}
            </button>
          ))}
        </div>

        <SummaryStrip
          tokens={data?.totals.totalTokens ?? 0}
          cost={data?.totals.costUsd ?? 0}
          calls={data?.totals.calls ?? 0}
          taskCount={data?.tasks.length ?? 0}
          unattributed={data?.unattributed.totalTokens ?? 0}
        />

        <div className="dash-body">
          {tab === 'tasks' && (
            <TasksPanel
              tasks={data?.tasks ?? []}
              unattributed={data?.unattributed.totalTokens ?? 0}
              activeTaskId={active.task?.taskId ?? null}
            />
          )}
          {tab === 'timeline' && <TimelinePanel daily={data?.daily ?? []} />}
          {tab === 'models' && <ModelsPanel modelShare={data?.modelShare ?? []} />}
          {tab === 'team' && <TeamPanel />}
          {!loading && (data?.tasks.length ?? 0) === 0 && tab === 'tasks' && <EmptyState />}
        </div>

        <footer className="dash-footer">
          <span className="f-item">
            <span className="f-dot" style={{ background: 'var(--green)' }} />
            <span className="f-lbl">Watcher</span>
            <span className="f-val">live</span>
          </span>
          <span className="f-sep" />
          <span className="f-item">
            <span className="f-lbl">DB</span>
            <span className="f-val">SQLite (local)</span>
          </span>
          <span className="f-sep" />
          <span className="f-item">
            <span className="f-lbl">Source</span>
            <span className="f-val">~/.claude/projects</span>
          </span>
          <span className="f-right">session resets in <span>5h</span></span>
        </footer>
      </div>
      <ToastHost />
    </div>
  );
}

function labelFor(t: Tab) {
  switch (t) {
    case 'tasks': return 'Tasks';
    case 'timeline': return 'Timeline';
    case 'models': return 'Models';
    case 'team': return 'Team';
  }
}

function HeaderMeter({ label, pct, value, tone }: { label: string; pct: number; value: string; tone: 's' | 'w' }) {
  return (
    <div className="dash-meter-g">
      <div className="dash-meter-meta">
        <span>{label}</span>
        <span>{value}</span>
      </div>
      <div className="dash-meter-track">
        <div className={`dash-meter-fill ${tone}`} style={{ width: `${Math.min(1, pct) * 100}%` }} />
      </div>
    </div>
  );
}

function QStat({ value, label, tone }: { value: string; label: string; tone: 'pulse' | 'green' | 'cyan' }) {
  return (
    <div className="dash-qs">
      <span className={`dash-qs-val ${tone}`}>{value}</span>
      <span className="dash-qs-lbl">{label}</span>
    </div>
  );
}

function SummaryStrip({
  tokens,
  cost,
  calls,
  taskCount,
  unattributed,
}: {
  tokens: number;
  cost: number;
  calls: number;
  taskCount: number;
  unattributed: number;
}) {
  return (
    <div className="dash-summary">
      <span className="sum-item"><span className="sum-dot" style={{ background: 'var(--pulse)' }} /><span className="sum-label">Tokens</span><span className="sum-val pulse">{formatTokens(tokens)}</span></span>
      <span className="sum-item"><span className="sum-dot" style={{ background: 'var(--green)' }} /><span className="sum-label">Cost</span><span className="sum-val green">{formatCostUsd(cost)}</span></span>
      <span className="sum-item"><span className="sum-dot" style={{ background: 'var(--cyan)' }} /><span className="sum-label">Calls</span><span className="sum-val cyan">{calls}</span></span>
      <span className="sum-item"><span className="sum-dot" style={{ background: 'var(--txt2)' }} /><span className="sum-label">Tasks</span><span className="sum-val txt">{taskCount}</span></span>
      <span className="sum-item"><span className="sum-dot" style={{ background: 'var(--txt3)' }} /><span className="sum-label">Unattributed</span><span className="sum-val txt">{formatTokens(unattributed)}</span></span>
    </div>
  );
}

function TasksPanel({
  tasks,
  unattributed,
  activeTaskId,
}: {
  tasks: TaskSnapshot[];
  unattributed: number;
  activeTaskId: string | null;
}) {
  const max = Math.max(1, ...tasks.map((t) => t.usage.totalTokens), unattributed);
  return (
    <>
      <div className="dash-col-head">
        <span className="col-h ticket">Ticket</span>
        <span className="col-h desc">Description</span>
        <span className="col-h bar">Usage</span>
        <span className="col-h toks">Tokens</span>
        <span className="col-h cost">Cost</span>
        <span className="col-h calls">Calls</span>
        <span className="col-h conf">Conf</span>
      </div>
      <div className="task-rows">
        {tasks.map((t) => {
          const id = t.taskId ?? '—';
          return (
            <div key={id} className={`t-row ${id === activeTaskId ? 'active' : ''}`}>
              <span className="t-ticket">{id}</span>
              <span className="t-desc">{t.taskName ?? t.branch ?? id}</span>
              {t.model && <span className="t-model-tag">{shortModel(t.model)}</span>}
              <div className="t-bar-w">
                <div className="t-bar-track">
                  <div className="t-bar-fill" style={{ width: `${(t.usage.totalTokens / max) * 100}%` }} />
                </div>
              </div>
              <span className="t-toks">{formatTokens(t.usage.totalTokens)}</span>
              <span className="t-cost">{formatCostUsd(t.usage.costUsd)}</span>
              <span className="t-calls">{t.usage.calls}</span>
              <span className="t-conf">
                <span className={`conf-pill ${t.confidence === 'high' ? 'hi' : t.confidence === 'medium' ? 'md' : 'lo'}`}>
                  {Math.round(t.confidenceScore * 100)}
                </span>
              </span>
            </div>
          );
        })}
        <div className="t-row">
          <span className="t-ticket dim">—</span>
          <span className="t-desc dim">Unattributed turns</span>
          <div className="t-bar-w">
            <div className="t-bar-track">
              <div className="t-bar-fill" style={{ width: `${(unattributed / max) * 100}%`, background: 'var(--txt4)' }} />
            </div>
          </div>
          <span className="t-toks">{formatTokens(unattributed)}</span>
          <span className="t-cost">—</span>
          <span className="t-calls">—</span>
          <span className="t-conf"><span className="conf-pill lo">N/A</span></span>
        </div>
      </div>
    </>
  );
}

function TimelinePanel({ daily }: { daily: { date: string; tokens: number; cost: number; calls: number }[] }) {
  const max = Math.max(1, ...daily.map((d) => d.tokens));
  const today = new Date().toISOString().slice(0, 10);
  return (
    <div className="dash-charts">
      <div className="chart-cell" style={{ borderRight: 'none' }}>
        <div className="chart-title">Tokens · last {daily.length} days</div>
        <div className="bar-chart">
          {daily.map((d) => (
            <div
              key={d.date}
              className={`bc-bar ${d.date === today ? 'today' : ''}`}
              style={{ height: `${(d.tokens / max) * 100}%` }}
              title={`${d.date}: ${formatTokens(d.tokens)}`}
            >
              <span className="bc-label">{d.date.slice(5)}</span>
            </div>
          ))}
          {daily.length === 0 && <div className="muted">No data yet.</div>}
        </div>
      </div>
    </div>
  );
}

function ModelsPanel({ modelShare }: { modelShare: { model: string; tokens: number; pct: number }[] }) {
  const colors = ['var(--pulse)', 'var(--cyan)', 'var(--green)', 'var(--amber)', 'var(--rose)'];
  let acc = 0;
  const segments = modelShare.slice(0, 5).map((m, i) => {
    const start = acc;
    const end = acc + m.pct * 360;
    acc = end;
    return `${colors[i]} ${start}deg ${end}deg`;
  });
  if (acc < 360) segments.push(`var(--s4) ${acc}deg 360deg`);

  return (
    <div className="dash-charts">
      <div className="chart-cell">
        <div className="chart-title">Model share · tokens</div>
        <div className="donut-wrap">
          <div className="donut" style={{ background: `conic-gradient(${segments.join(', ')})` }} />
          <div className="donut-legend">
            {modelShare.slice(0, 5).map((m, i) => (
              <div key={m.model} className="dl-item">
                <span className="dl-dot" style={{ background: colors[i] }} />
                <span className="dl-name" title={m.model}>{shortModel(m.model)}</span>
                <span className="dl-pct">{formatPct(m.pct)}</span>
              </div>
            ))}
            {modelShare.length === 0 && <div className="muted">No data yet.</div>}
          </div>
        </div>
      </div>
      <div className="chart-cell">
        <div className="chart-title">By absolute tokens</div>
        {modelShare.map((m, i) => (
          <div className="dl-item" key={m.model} style={{ marginBottom: 6 }}>
            <span className="dl-dot" style={{ background: colors[i] ?? 'var(--txt3)' }} />
            <span className="dl-name">{shortModel(m.model)}</span>
            <span className="dl-pct">{formatTokens(m.tokens)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function TeamPanel() {
  return (
    <div className="chart-cell" style={{ padding: '24px 18px' }}>
      <div className="chart-title">Team — coming soon</div>
      <div className="muted" style={{ fontFamily: 'var(--sans)', fontSize: 11 }}>
        Multi-developer rollups land in v0.2 once the team-sync service ships.
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div style={{ padding: '40px 20px', textAlign: 'center' }}>
      <div style={{ fontFamily: 'var(--sans)', fontSize: 13, color: 'var(--txt2)', marginBottom: 6 }}>
        No usage recorded yet.
      </div>
      <div style={{ fontFamily: 'var(--mono)', fontSize: 10, color: 'var(--txt3)' }}>
        Pulse will populate this view as soon as Claude Code writes new
        session transcripts to <code>~/.claude/projects</code>.
      </div>
    </div>
  );
}

function shortModel(m: string): string {
  return m.replace('claude-', '').replace('-20', "'").replace(/-(\d+)$/, '.$1');
}
