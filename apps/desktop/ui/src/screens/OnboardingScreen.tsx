import { useEffect, useState } from 'react';
import type { OnboardingStatus } from '@pulse/types';
import { getOnboardingStatus, isTauri, openDashboard } from '../lib/tauri';
import { PulseLogoMark } from '../components/PulseLogo';
import { getCurrentWindow } from '@tauri-apps/api/window';

const STEPS = [
  {
    title: 'Locate ~/.claude/projects',
    body: "Pulse reads the JSONL session transcripts Claude Code already writes — no interception.",
  },
  {
    title: 'Index existing transcripts',
    body: 'Initial scan parses every line into the local SQLite store. This runs once.',
  },
  {
    title: 'Watch for new sessions',
    body: 'A debounced FS watcher keeps the dashboard live as Claude Code writes new turns.',
  },
  {
    title: 'Attribute to Jira tasks',
    body: 'Git branch → regex → Jira ID. Tune the pattern in Settings → Jira attribution.',
  },
];

export function OnboardingScreen() {
  const [status, setStatus] = useState<OnboardingStatus | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    let timer: ReturnType<typeof setInterval> | null = null;
    const tick = async () => {
      try {
        const s = await getOnboardingStatus();
        if (alive) setStatus(s);
      } catch (err) {
        console.warn('onboarding status', err);
      }
    };
    tick();
    timer = setInterval(tick, 1500);
    return () => {
      alive = false;
      if (timer) clearInterval(timer);
    };
  }, []);

  const completed = countComplete(status);

  return (
    <div className="window">
      <div className="onboarding">
        <header className="settings-header">
          <PulseLogoMark size={26} />
          <span className="settings-title">Welcome to Pulse</span>
        </header>

        <div className="settings-body" style={{ gap: 22 }}>
          <div style={{ fontFamily: 'var(--sans)', fontSize: 13, color: 'var(--txt2)', lineHeight: 1.5 }}>
            One install. One tray icon. Complete AI usage visibility.
            <br />
            No workflow change required.
          </div>

          <div>
            {STEPS.map((s, i) => {
              const done = i < completed;
              const current = i === completed;
              return (
                <div key={i} className={`onb-step ${done ? 'done' : current ? 'current' : ''}`}>
                  <span className="num">{i + 1}</span>
                  <div className="text">
                    <div style={{ color: 'inherit', fontWeight: 600 }}>{s.title}</div>
                    <div className="muted" style={{ fontSize: 10, fontFamily: 'var(--mono)', marginTop: 2 }}>
                      {s.body}
                    </div>
                    {i === 0 && status?.claudeDirFound && (
                      <div className="onb-detail">{status.claudeDirPath}</div>
                    )}
                    {i === 1 && status && (
                      <div className="onb-detail">{status.sessionsDiscovered} sessions found</div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          <div style={{ display: 'flex', gap: 8, marginTop: 'auto' }}>
            <button
              className="btn primary"
              onClick={async () => {
                await openDashboard();
                if (isTauri()) await getCurrentWindow().hide();
              }}
            >
              Open dashboard
            </button>
            <button
              className="btn"
              onClick={async () => {
                if (isTauri()) await getCurrentWindow().hide();
              }}
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function countComplete(status: OnboardingStatus | null): number {
  if (!status) return 0;
  let n = 0;
  if (status.claudeDirFound) n++;
  if (status.sessionsDiscovered > 0) n++;
  if (status.ingestComplete) n++;
  return n;
}
