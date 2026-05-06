import type { ActiveTask } from '@pulse/types';
import { formatPct, formatTokens, formatCostUsd } from '@pulse/types';
import { PulseLogo } from './PulseLogo';

export function TrayBar({ active }: { active: ActiveTask }) {
  const stateClass =
    active.state === 'crit' ? 'crit'
    : active.state === 'warn' ? 'warn'
    : active.state === 'idle' ? 'idle'
    : '';

  const tokens = active.task?.usage.totalTokens ?? 0;
  const cost = active.task?.usage.costUsd ?? 0;
  const ticket = active.task?.taskId ?? '—';

  return (
    <div className={`tray-bar ${stateClass}`}>
      <span className="tray-logo">
        <PulseLogo size={14} />
      </span>
      <div className="tray-meters">
        <div className="tray-track">
          <div className="tray-fill session" style={{ width: `${Math.min(1, active.sessionUsedPct) * 100}%` }} />
        </div>
        <div className="tray-track">
          <div className="tray-fill weekly" style={{ width: `${Math.min(1, active.weeklyUsedPct) * 100}%` }} />
        </div>
      </div>
      <span className="tray-div" />
      <span className="tray-ticket">{ticket}</span>
      <span className="tray-tokens">{formatTokens(tokens)}</span>
      <span className="tray-cost">{formatCostUsd(cost)}</span>
      <span className="tray-dot" title={`Session ${formatPct(active.sessionUsedPct)}`} />
    </div>
  );
}
