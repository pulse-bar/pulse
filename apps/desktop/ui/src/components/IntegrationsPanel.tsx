import { useCallback, useEffect, useState } from 'react';
import type { PluginManifest, PluginStatus } from '@pulse/types';
import { listPluginStatuses, listPlugins } from '../lib/tauri';
import { ErrorBoundary } from './ErrorBoundary';
import { IntegrationCard } from './IntegrationCard';
import { JiraDetail } from './integrations/JiraDetail';

type View = { kind: 'list' } | { kind: 'detail'; pluginId: string };

export function IntegrationsPanel() {
  const [view, setView] = useState<View>({ kind: 'list' });
  const [plugins, setPlugins] = useState<PluginManifest[]>([]);
  const [statuses, setStatuses] = useState<Record<string, PluginStatus>>({});
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [m, s] = await Promise.all([listPlugins(), listPluginStatuses()]);
      const byId: Record<string, PluginStatus> = {};
      for (const status of s) byId[status.pluginId] = status;
      setPlugins(m);
      setStatuses(byId);
      setLoadError(null);
    } catch (err) {
      console.warn('plugin load failed', err);
      setLoadError(typeof err === 'string' ? err : err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  // Background poll only on list view; detail view does its own thing.
  useEffect(() => {
    refresh();
    if (view.kind !== 'list') return;
    const t = setInterval(refresh, 15_000);
    return () => clearInterval(t);
  }, [refresh, view.kind]);

  if (view.kind === 'detail') {
    const manifest = plugins.find((p) => p.id === view.pluginId);
    return (
      <ErrorBoundary label={`Integrations · ${view.pluginId}`}>
        <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <button
            onClick={() => setView({ kind: 'list' })}
            style={{
              alignSelf: 'flex-start',
              marginBottom: 14,
              padding: '6px 10px',
              borderRadius: 4,
              background: 'var(--s2)',
              border: '1px solid var(--b1)',
              color: 'var(--txt2)',
              fontFamily: 'var(--sans)',
              fontSize: 11,
              cursor: 'pointer',
            }}
          >
            ← All integrations
          </button>
          {!manifest && (
            <div style={{ fontFamily: 'var(--mono)', fontSize: 10, color: 'var(--txt3)' }}>
              Loading…
            </div>
          )}
          {manifest && view.pluginId === 'jira' && (
            <JiraDetail
              manifest={manifest}
              status={statuses[view.pluginId]}
              onChanged={refresh}
            />
          )}
          {manifest && view.pluginId !== 'jira' && (
            <div
              style={{
                background: 'var(--s2)',
                border: '1px dashed var(--b1)',
                borderRadius: 8,
                padding: 16,
                fontFamily: 'var(--sans)',
                fontSize: 12,
                color: 'var(--txt2)',
              }}
            >
              The {manifest.displayName} integration is wired into the registry but its
              configuration UI hasn't shipped yet.
            </div>
          )}
        </div>
      </ErrorBoundary>
    );
  }

  return (
    <ErrorBoundary label="Integrations">
      <div
        style={{
          fontFamily: 'var(--sans)',
          fontSize: 12,
          color: 'var(--txt2)',
          marginBottom: 14,
          lineHeight: 1.5,
        }}
      >
        Connect Pulse to your task tracker, source-control, and communication tools.
        Credentials are stored in your OS keychain — never in plaintext config.
      </div>

      {loadError && (
        <div
          style={{
            background: 'var(--red-d)',
            border: '1px solid rgba(220, 70, 70, 0.3)',
            borderRadius: 6,
            padding: '10px 12px',
            marginBottom: 12,
            fontFamily: 'var(--mono)',
            fontSize: 10,
            color: 'var(--red)',
          }}
        >
          Couldn't load plugins: {loadError}
        </div>
      )}

      {loading && plugins.length === 0 ? (
        <div style={{ fontFamily: 'var(--mono)', fontSize: 10, color: 'var(--txt3)' }}>
          Loading integrations…
        </div>
      ) : (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))',
            gap: 12,
          }}
        >
          {plugins.map((m) => (
            <IntegrationCard
              key={m.id}
              manifest={m}
              status={statuses[m.id]}
              onClick={() => setView({ kind: 'detail', pluginId: m.id })}
            />
          ))}
        </div>
      )}
    </ErrorBoundary>
  );
}
