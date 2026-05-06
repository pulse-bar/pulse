import { useActiveTask } from '../hooks/useActiveTask';
import { formatCostUsd, formatPct, formatTokens } from '@pulse/types';
import { openDashboard, openSettingsWindow } from '../lib/tauri';

export function PopoverScreen() {
  const active = useActiveTask();
  const task = active.task;
  const usage = task?.usage;

  const stateBadge =
    active.state === 'crit' ? { cls: 'crit', label: 'CRIT' }
    : active.state === 'warn' ? { cls: 'warn', label: 'WARN' }
    : active.state === 'idle' ? { cls: 'model', label: 'IDLE' }
    : { cls: 'active', label: 'ACTIVE' };

  return (
    <div className="window window--popover">
      <div className="popover">
        <div className="pop-header">
          <div className="pop-task-row">
            <span className="pop-task-id">{task?.taskId ?? 'UNATTRIBUTED'}</span>
            <span className={`pop-badge ${stateBadge.cls}`}>{stateBadge.label}</span>
          </div>
          <div className="pop-task-name">{task?.taskName ?? 'No active session detected'}</div>
          <div className="pop-branch">
            <svg viewBox="0 0 16 16" width="9" height="9" fill="currentColor" aria-hidden>
              <path d="M5 3a2 2 0 1 1 0 4 2 2 0 0 1 0-4Zm6 0a2 2 0 1 1 0 4 2 2 0 0 1 0-4Zm-6 6.5a2 2 0 1 1 0 4 2 2 0 0 1 0-4ZM6 7v2a2 2 0 0 0 2 2h1a2 2 0 0 1 2 2" />
            </svg>
            {task?.branch ?? '—'}
          </div>
        </div>

        <div className="pop-meters">
          <Meter
            name="Session window"
            pct={active.sessionUsedPct}
            value={`${formatPct(active.sessionUsedPct)} · ${formatTokens(usage?.totalTokens ?? 0)}`}
            state={active.state}
          />
          <Meter
            name="Weekly quota"
            pct={active.weeklyUsedPct}
            value={formatPct(active.weeklyUsedPct)}
            weekly
          />
        </div>

        <div className="pop-grid">
          <Stat
            label="Tokens"
            value={formatTokens(usage?.totalTokens ?? 0)}
            sub={`${formatTokens(usage?.inputTokens ?? 0)} in · ${formatTokens(usage?.outputTokens ?? 0)} out`}
            tone="pulse"
          />
          <Stat label="Cost" value={formatCostUsd(usage?.costUsd ?? 0)} sub="this session" tone="green" />
          <Stat label="Calls" value={String(usage?.calls ?? 0)} sub={task?.model ?? '—'} tone="cyan" />
        </div>

        <div className="pop-cache">
          <CacheBar label="Cache hit" pct={usage?.cacheHitRate ?? 0} tone="hit" />
          <CacheBar label="Fresh inp" pct={1 - (usage?.cacheHitRate ?? 0)} tone="inp" />
        </div>

        <div className="pop-footer">
          <div className="pop-conf">
            <span
              className={`pop-conf-dot ${
                task?.confidence === 'high' ? '' : task?.confidence === 'medium' ? 'medium' : 'low'
              }`}
            />
            <span className="pop-conf-text">Confidence</span>
            <span className="pop-conf-pct">{formatPct(task?.confidenceScore ?? 0)}</span>
          </div>
          <div className="pop-actions">
            <button className="pop-action" onClick={() => openSettingsWindow()}>Settings</button>
            <button className="pop-action" onClick={() => openDashboard()}>Dashboard ↗</button>
          </div>
        </div>
      </div>
    </div>
  );
}

function Meter({
  name,
  pct,
  value,
  state,
  weekly,
}: {
  name: string;
  pct: number;
  value: string;
  state?: 'normal' | 'warn' | 'crit' | 'idle';
  weekly?: boolean;
}) {
  const fillCls = weekly ? 'weekly' : state === 'crit' ? 'crit' : state === 'warn' ? 'warn' : '';
  const valCls = state === 'crit' ? 'crit' : state === 'warn' ? 'warn' : '';
  return (
    <div className="pop-meter-row">
      <div className="pop-meter-label">
        <span className="pop-meter-name">{name}</span>
        <span className={`pop-meter-val ${valCls}`}>{value}</span>
      </div>
      <div className="pop-track">
        <div className={`pop-fill ${fillCls}`} style={{ width: `${Math.min(1, pct) * 100}%` }} />
      </div>
    </div>
  );
}

function Stat({
  label,
  value,
  sub,
  tone,
}: {
  label: string;
  value: string;
  sub: string;
  tone: 'pulse' | 'green' | 'cyan';
}) {
  return (
    <div className="pop-cell">
      <span className="pop-cell-label">{label}</span>
      <span className={`pop-cell-val ${tone}`}>{value}</span>
      <span className="pop-cell-sub">{sub}</span>
    </div>
  );
}

function CacheBar({ label, pct, tone }: { label: string; pct: number; tone: 'hit' | 'inp' }) {
  return (
    <div className="pop-cache-row">
      <span className="pop-cache-label">{label}</span>
      <div className="pop-cache-track">
        <div className={`pop-cache-fill ${tone}`} style={{ width: `${Math.min(1, pct) * 100}%` }} />
      </div>
      <span className="pop-cache-val">{formatPct(pct)}</span>
    </div>
  );
}
