import { useState } from 'react';
import type { Settings } from '@pulse/types';
import { useSettings } from '../hooks/useSettings';
import { resetDatabase, triggerFullRescan } from '../lib/tauri';
import { PulseLogoMark } from '../components/PulseLogo';

export function SettingsScreen() {
  const { settings, save, loading } = useSettings();
  const [dirty, setDirty] = useState<Settings | null>(null);
  const [busy, setBusy] = useState(false);
  const draft = dirty ?? settings;

  if (loading) return <div className="window" />;

  const update = (patch: Partial<Settings>) => setDirty({ ...(dirty ?? settings), ...patch });

  return (
    <div className="window">
      <div className="settings">
        <header className="settings-header">
          <PulseLogoMark size={24} />
          <span className="settings-title">Settings</span>
          <span style={{ marginLeft: 'auto' }}>
            <button
              className="btn primary"
              disabled={!dirty}
              onClick={async () => {
                if (!dirty) return;
                await save(dirty);
                setDirty(null);
              }}
            >
              Save
            </button>
          </span>
        </header>

        <div className="settings-body">
          <Section title="Appearance">
            <Row label="Theme">
              <select
                value={draft.appearance}
                onChange={(e) =>
                  update({ appearance: e.target.value as 'dark' | 'light' | 'auto' })
                }
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
                <option value="auto">Auto</option>
              </select>
            </Row>
            <Toggle label="Start at login" value={draft.startAtLogin} onChange={(v) => update({ startAtLogin: v })} />
          </Section>

          <Section title="Jira attribution">
            <Row label="Jira base URL">
              <input
                type="text"
                placeholder="https://company.atlassian.net"
                value={draft.jiraBaseUrl ?? ''}
                onChange={(e) => update({ jiraBaseUrl: e.target.value || null })}
              />
            </Row>
            <Row label="Project keys (comma)">
              <input
                type="text"
                placeholder="PROJ, WEB"
                value={draft.jiraProjectKeys.join(', ')}
                onChange={(e) =>
                  update({
                    jiraProjectKeys: e.target.value
                      .split(',')
                      .map((s) => s.trim().toUpperCase())
                      .filter(Boolean),
                  })
                }
              />
            </Row>
            <Row label="Branch regex">
              <input
                type="text"
                value={draft.branchRegex}
                onChange={(e) => update({ branchRegex: e.target.value })}
              />
            </Row>
          </Section>

          <Section title="Budgets & thresholds">
            <Row label="Session budget (tokens)">
              <input
                type="number"
                value={draft.sessionTokenBudget}
                onChange={(e) => update({ sessionTokenBudget: Number(e.target.value) || 0 })}
              />
            </Row>
            <Row label="Weekly budget (tokens)">
              <input
                type="number"
                value={draft.weeklyTokenBudget}
                onChange={(e) => update({ weeklyTokenBudget: Number(e.target.value) || 0 })}
              />
            </Row>
            <Row label="Warn at %">
              <input
                type="number"
                step="0.01"
                value={draft.warnThresholdPct}
                onChange={(e) => update({ warnThresholdPct: Number(e.target.value) })}
              />
            </Row>
            <Row label="Critical at %">
              <input
                type="number"
                step="0.01"
                value={draft.critThresholdPct}
                onChange={(e) => update({ critThresholdPct: Number(e.target.value) })}
              />
            </Row>
          </Section>

          <Section title="Notifications">
            <Toggle label="Warn at 78%" value={draft.notifyOnWarn} onChange={(v) => update({ notifyOnWarn: v })} />
            <Toggle label="Critical at 92%" value={draft.notifyOnCrit} onChange={(v) => update({ notifyOnCrit: v })} />
            <Toggle label="Daily summary at 5pm" value={draft.notifyDailySummary} onChange={(v) => update({ notifyDailySummary: v })} />
          </Section>

          <Section title="Providers">
            <ProviderCard name="Claude Code" status="connected" />
            <ProviderCard name="Codex CLI" status="soon" />
            <ProviderCard name="Gemini CLI" status="soon" />
          </Section>

          <Section title="Maintenance">
            <Row label="Re-scan all transcripts">
              <button
                className="btn"
                disabled={busy}
                onClick={async () => {
                  setBusy(true);
                  try {
                    await triggerFullRescan();
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                {busy ? 'Scanning…' : 'Scan now'}
              </button>
            </Row>
            <Row label="Clear local database">
              <button
                className="btn"
                onClick={async () => {
                  if (confirm('Clear all locally stored Pulse data?')) {
                    await resetDatabase();
                  }
                }}
              >
                Reset
              </button>
            </Row>
          </Section>
        </div>
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="settings-section">
      <h4>{title}</h4>
      {children}
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="settings-row">
      <label>{label}</label>
      {children}
    </div>
  );
}

function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <Row label={label}>
      <button className={`toggle ${value ? 'on' : ''}`} aria-pressed={value} onClick={() => onChange(!value)} />
    </Row>
  );
}

function ProviderCard({ name, status }: { name: string; status: 'connected' | 'soon' }) {
  return (
    <div className={`provider-card ${status === 'connected' ? 'connected' : 'coming-soon'}`}>
      <span className="provider-name">{name}</span>
      <span className={`provider-status ${status === 'connected' ? 'connected' : 'soon'}`}>
        {status === 'connected' ? 'Connected' : 'Soon'}
      </span>
    </div>
  );
}
