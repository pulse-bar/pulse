import { useState } from 'react';
import type { Settings } from '@pulse/types';
import { useSettings } from '../hooks/useSettings';
import { resetDatabase, runEnrichmentNow, triggerFullRescan } from '../lib/tauri';
import { PulseLogoMark } from '../components/PulseLogo';
import { IntegrationsPanel } from '../components/IntegrationsPanel';
import { ErrorBoundary } from '../components/ErrorBoundary';

type Section = 'general' | 'integrations' | 'maintenance';

export function SettingsScreen() {
  const { settings, save, loading } = useSettings();
  const [section, setSection] = useState<Section>('general');
  const [dirty, setDirty] = useState<Settings | null>(null);
  const draft = dirty ?? settings;

  if (loading) return <div className="window" />;

  const update = (patch: Partial<Settings>) => setDirty({ ...(dirty ?? settings), ...patch });
  const persistGeneral = async () => {
    if (!dirty) return;
    await save(dirty);
    setDirty(null);
  };

  return (
    <div className="window">
      <div className="settings settings--shell">
        <header className="settings-header">
          <PulseLogoMark size={24} />
          <span className="settings-title">Settings</span>
          {section === 'general' && dirty && (
            <span style={{ marginLeft: 'auto' }}>
              <button className="btn primary" onClick={persistGeneral}>
                Save changes
              </button>
            </span>
          )}
        </header>

        <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
          <nav
            style={{
              width: 160,
              flexShrink: 0,
              borderRight: '1px solid var(--b1)',
              padding: '14px 8px',
              display: 'flex',
              flexDirection: 'column',
              gap: 2,
            }}
          >
            <NavItem active={section === 'general'} onClick={() => setSection('general')}>
              General
            </NavItem>
            <NavItem
              active={section === 'integrations'}
              onClick={() => setSection('integrations')}
            >
              Integrations
            </NavItem>
            <NavItem
              active={section === 'maintenance'}
              onClick={() => setSection('maintenance')}
            >
              Maintenance
            </NavItem>
          </nav>

          <div
            style={{
              flex: 1,
              overflowY: 'auto',
              padding: '18px 22px 24px',
              minWidth: 0,
            }}
          >
            <ErrorBoundary label={`Settings · ${section}`}>
              {section === 'general' && <GeneralSection draft={draft} update={update} />}
              {section === 'integrations' && <IntegrationsPanel />}
              {section === 'maintenance' && <MaintenanceSection />}
            </ErrorBoundary>
          </div>
        </div>
      </div>
    </div>
  );
}

function NavItem({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        textAlign: 'left',
        padding: '7px 10px',
        borderRadius: 4,
        background: active ? 'var(--pulse-d)' : 'transparent',
        color: active ? 'var(--pulse)' : 'var(--txt2)',
        fontFamily: 'var(--sans)',
        fontSize: 11,
        fontWeight: active ? 600 : 500,
        cursor: 'pointer',
        transition: 'background 0.15s, color 0.15s',
      }}
    >
      {children}
    </button>
  );
}

function GeneralSection({
  draft,
  update,
}: {
  draft: Settings;
  update: (patch: Partial<Settings>) => void;
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
      <Section title="Appearance">
        <Row label="Theme">
          <select
            value={draft.appearance}
            onChange={(e) =>
              update({ appearance: e.target.value as Settings['appearance'] })
            }
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="auto">Auto · follow OS</option>
          </select>
        </Row>
        <Toggle
          label="Start at login"
          value={draft.startAtLogin}
          onChange={(v) => update({ startAtLogin: v })}
        />
      </Section>

      <Section title="Attribution">
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
        <Toggle
          label="Warn at threshold"
          value={draft.notifyOnWarn}
          onChange={(v) => update({ notifyOnWarn: v })}
        />
        <Toggle
          label="Critical at threshold"
          value={draft.notifyOnCrit}
          onChange={(v) => update({ notifyOnCrit: v })}
        />
        <Toggle
          label="Daily summary at 5pm"
          value={draft.notifyDailySummary}
          onChange={(v) => update({ notifyDailySummary: v })}
        />
      </Section>

      <Section title="Enrichment">
        <Toggle
          label="Enrich tickets in background"
          value={draft.enrichmentEnabled}
          onChange={(v) => update({ enrichmentEnabled: v })}
        />
        <Row label="Cache TTL (hours)">
          <input
            type="number"
            step="1"
            value={Math.round(draft.enrichmentCacheTtlSecs / 3600)}
            onChange={(e) =>
              update({
                enrichmentCacheTtlSecs: Math.max(1, Number(e.target.value)) * 3600,
              })
            }
          />
        </Row>
      </Section>
    </div>
  );
}

function MaintenanceSection() {
  const [busy, setBusy] = useState(false);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
      <Section title="Local data">
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
        <Row label="Run enrichment now">
          <button
            className="btn"
            onClick={async () => {
              await runEnrichmentNow();
            }}
          >
            Run now
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
            Reset database
          </button>
        </Row>
      </Section>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h4
        style={{
          fontFamily: 'var(--sans)',
          fontSize: 9,
          color: 'var(--txt3)',
          textTransform: 'uppercase',
          letterSpacing: '0.12em',
          fontWeight: 600,
          marginBottom: 8,
        }}
      >
        {title}
      </h4>
      <div style={{ display: 'flex', flexDirection: 'column' }}>{children}</div>
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
      <button
        className={`toggle ${value ? 'on' : ''}`}
        aria-pressed={value}
        onClick={() => onChange(!value)}
      />
    </Row>
  );
}
