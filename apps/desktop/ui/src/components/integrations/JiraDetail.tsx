import { useEffect, useState } from 'react';
import type {
  JiraAuthKind,
  JiraSite,
  PluginManifest,
  PluginStatus,
} from '@pulse/types';
import {
  connectJiraOauth,
  deleteJiraSite,
  deleteJiraToken,
  getSettings,
  jiraTokenPresent,
  onOauthResult,
  storeJiraToken,
  testPluginInstance,
  upsertJiraSite,
} from '../../lib/tauri';

type SiteWithDraft = JiraSite & { _tokenStored: boolean };

function newSite(): JiraSite {
  return {
    id: crypto.randomUUID(),
    label: 'New Jira site',
    baseUrl: '',
    projectKeys: [],
    authKind: 'oauth-2',
    email: '',
    oauthClientId: '',
    enabled: true,
  };
}

export function JiraDetail({
  manifest,
  status,
  onChanged,
}: {
  manifest: PluginManifest;
  status: PluginStatus | undefined;
  onChanged: () => void;
}) {
  const [sites, setSites] = useState<SiteWithDraft[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const reload = async () => {
    try {
      const s = await getSettings();
      const withTokens = await Promise.all(
        (s.jira?.sites ?? []).map(async (site) => {
          let stored = false;
          try {
            stored = await jiraTokenPresent(site.id);
          } catch {
            stored = false;
          }
          return { ...site, _tokenStored: stored };
        }),
      );
      setSites(withTokens);
      setLoadError(null);
    } catch (err) {
      console.warn('jira reload failed', err);
      setLoadError(
        typeof err === 'string' ? err : err instanceof Error ? err.message : String(err),
      );
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    reload().catch(console.warn);
    let unlisten: (() => void) | null = null;
    (async () => {
      unlisten = await onOauthResult(async () => {
        await reload();
        onChanged();
      });
    })();
    return () => unlisten?.();
  }, []);

  const update = async (idx: number, patch: Partial<JiraSite>) => {
    const copy = sites.slice();
    copy[idx] = { ...copy[idx], ...patch };
    setSites(copy);
    const { _tokenStored, ...site } = copy[idx];
    await upsertJiraSite(site);
    onChanged();
  };

  const remove = async (idx: number) => {
    const site = sites[idx];
    if (site._tokenStored) {
      await deleteJiraToken(site.id);
    }
    const copy = sites.slice();
    copy.splice(idx, 1);
    setSites(copy);
    await deleteJiraSite(site.id);
    onChanged();
  };

  const add = async () => {
    const site = newSite();
    setSites([...sites, { ...site, _tokenStored: false }]);
    await upsertJiraSite(site);
    onChanged();
  };

  if (loading) {
    return (
      <div style={{ fontFamily: 'var(--mono)', fontSize: 10, color: 'var(--txt3)' }}>
        Loading…
      </div>
    );
  }

  return (
    <div>
      <Header manifest={manifest} status={status} />
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
          Couldn't load Jira config: {loadError}
        </div>
      )}

      {sites.length === 0 && (
        <div
          style={{
            fontFamily: 'var(--sans)',
            fontSize: 12,
            color: 'var(--txt2)',
            background: 'var(--s2)',
            border: '1px dashed var(--b1)',
            borderRadius: 8,
            padding: 16,
            textAlign: 'center',
            margin: '16px 0',
          }}
        >
          No Jira sites yet. Add your first site below — most users connect with OAuth in
          one click. <a href="#" style={{ color: 'var(--pulse)' }}>How to register an OAuth app →</a>
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        {sites.map((site, idx) => (
          <SiteCard
            key={site.id}
            site={site}
            onUpdate={(patch) => update(idx, patch)}
            onRemove={() => remove(idx)}
            onTokenChanged={reload}
          />
        ))}
      </div>

      <div style={{ marginTop: 14, display: 'flex', gap: 8 }}>
        <button className="btn primary" onClick={add}>
          + Add Jira site
        </button>
      </div>
    </div>
  );
}

function Header({
  manifest,
  status,
}: {
  manifest: PluginManifest;
  status: PluginStatus | undefined;
}) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        gap: 14,
        paddingBottom: 16,
        marginBottom: 16,
        borderBottom: '1px solid var(--b1)',
      }}
    >
      <span
        style={{
          width: 44,
          height: 44,
          borderRadius: 8,
          background: 'var(--s3)',
          border: '1px solid var(--b1)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily: 'var(--sans)',
          fontWeight: 700,
          fontSize: 18,
          color: 'var(--pulse)',
        }}
      >
        J
      </span>
      <div style={{ flex: 1 }}>
        <div
          style={{
            fontFamily: 'var(--sans)',
            fontSize: 16,
            fontWeight: 700,
            color: 'var(--txt)',
            marginBottom: 2,
          }}
        >
          {manifest.displayName}
        </div>
        <div
          style={{
            fontFamily: 'var(--mono)',
            fontSize: 9,
            color: 'var(--txt3)',
            textTransform: 'uppercase',
            letterSpacing: '0.06em',
          }}
        >
          {manifest.vendor}
        </div>
        <div
          style={{
            fontFamily: 'var(--sans)',
            fontSize: 12,
            color: 'var(--txt2)',
            marginTop: 8,
            lineHeight: 1.5,
          }}
        >
          {manifest.description}
        </div>
        {status?.error && (
          <div
            style={{
              marginTop: 10,
              fontFamily: 'var(--mono)',
              fontSize: 10,
              color: 'var(--red)',
              background: 'var(--red-d)',
              padding: '8px 10px',
              borderRadius: 4,
              border: '1px solid rgba(220, 70, 70, 0.25)',
            }}
          >
            {status.error}
          </div>
        )}
      </div>
    </div>
  );
}

function SiteCard({
  site,
  onUpdate,
  onRemove,
  onTokenChanged,
}: {
  site: SiteWithDraft;
  onUpdate: (patch: Partial<JiraSite>) => void;
  onRemove: () => void;
  onTokenChanged: () => void;
}) {
  const [token, setToken] = useState('');
  const [connecting, setConnecting] = useState(false);
  const [testStatus, setTestStatus] = useState<{
    state: 'idle' | 'busy' | 'ok' | 'error';
    msg?: string;
  }>({ state: 'idle' });
  const [connectError, setConnectError] = useState<string | null>(null);

  const saveToken = async () => {
    if (!token.trim()) return;
    if (site.authKind !== 'bearer' && site.authKind !== 'basic') return;
    await storeJiraToken(site.id, site.authKind, token.trim());
    setToken('');
    onTokenChanged();
  };
  const removeToken = async () => {
    await deleteJiraToken(site.id);
    onTokenChanged();
  };
  const test = async () => {
    setTestStatus({ state: 'busy' });
    try {
      await testPluginInstance('jira', site.id);
      setTestStatus({ state: 'ok', msg: 'Connected' });
    } catch (err) {
      setTestStatus({ state: 'error', msg: String(err) });
    }
  };
  const connect = async () => {
    setConnectError(null);
    setConnecting(true);
    try {
      if (!site.oauthClientId?.trim()) {
        setConnectError(
          'OAuth client ID is required. Register an Atlassian OAuth 2.0 app once — see the setup guide.',
        );
        setConnecting(false);
        return;
      }
      await connectJiraOauth(site.id, site.oauthClientId.trim());
    } catch (err) {
      setConnectError(String(err));
      setConnecting(false);
    }
  };

  // Reset connecting state when token state flips to true after onOauthResult.
  useEffect(() => {
    if (site._tokenStored) setConnecting(false);
  }, [site._tokenStored]);

  return (
    <div
      style={{
        background: 'var(--s2)',
        border: '1px solid var(--b1)',
        borderRadius: 8,
        padding: 14,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <input
          type="text"
          value={site.label}
          onChange={(e) => onUpdate({ label: e.target.value })}
          placeholder="Site label (e.g. Acme Engineering)"
          style={inputStyle({ flex: 1 })}
        />
        <button
          className={`toggle ${site.enabled ? 'on' : ''}`}
          aria-pressed={site.enabled}
          onClick={() => onUpdate({ enabled: !site.enabled })}
          title={site.enabled ? 'Enabled' : 'Disabled'}
        />
        <button className="btn" onClick={onRemove} title="Remove site">
          ×
        </button>
      </div>

      <Row label="Base URL">
        <input
          type="text"
          value={site.baseUrl}
          onChange={(e) => onUpdate({ baseUrl: e.target.value })}
          placeholder="https://yourcompany.atlassian.net"
          style={inputStyle()}
        />
      </Row>

      <Row label="Project keys">
        <input
          type="text"
          value={site.projectKeys.join(', ')}
          onChange={(e) =>
            onUpdate({
              projectKeys: e.target.value
                .split(',')
                .map((s) => s.trim().toUpperCase())
                .filter(Boolean),
            })
          }
          placeholder="PROJ, WEB"
          style={inputStyle()}
        />
      </Row>

      <Row label="Authentication">
        <select
          value={site.authKind}
          onChange={(e) => onUpdate({ authKind: e.target.value as JiraAuthKind })}
          style={inputStyle()}
        >
          <option value="oauth-2">OAuth 2.0 — recommended</option>
          <option value="basic">Basic — email + API token</option>
          <option value="bearer">Bearer — Personal Access Token</option>
          <option value="none">None — public</option>
        </select>
      </Row>

      {site.authKind === 'oauth-2' && (
        <>
          <Row label="OAuth client ID">
            <input
              type="text"
              value={site.oauthClientId ?? ''}
              onChange={(e) => onUpdate({ oauthClientId: e.target.value || null })}
              placeholder="From developer.atlassian.com"
              style={inputStyle()}
            />
          </Row>

          <div
            style={{
              fontFamily: 'var(--mono)',
              fontSize: 9,
              color: 'var(--txt3)',
              padding: '4px 0 0 138px',
            }}
          >
            One-time setup: register an OAuth 2.0 (3LO) app at{' '}
            <span style={{ color: 'var(--cyan)' }}>developer.atlassian.com/console/myapps</span>{' '}
            and add{' '}
            <span style={{ color: 'var(--cyan)' }}>http://127.0.0.1:19834/callback</span>{' '}
            as a callback URL. Pulse handles the rest.
          </div>

          <div
            style={{
              display: 'flex',
              gap: 8,
              alignItems: 'center',
              paddingTop: 6,
            }}
          >
            {site._tokenStored ? (
              <>
                <span
                  style={{
                    fontFamily: 'var(--mono)',
                    fontSize: 10,
                    color: 'var(--green)',
                  }}
                >
                  ✓ Connected · tokens in keychain
                </span>
                <button className="btn" onClick={removeToken}>
                  Disconnect
                </button>
              </>
            ) : (
              <button
                className="btn primary"
                onClick={connect}
                disabled={connecting || !site.oauthClientId?.trim() || !site.baseUrl.trim()}
              >
                {connecting ? 'Waiting for browser…' : 'Connect with Atlassian →'}
              </button>
            )}
          </div>

          {connectError && (
            <div
              style={{
                fontFamily: 'var(--mono)',
                fontSize: 10,
                color: 'var(--red)',
                background: 'var(--red-d)',
                padding: '6px 8px',
                borderRadius: 4,
              }}
            >
              {connectError}
            </div>
          )}
        </>
      )}

      {site.authKind === 'basic' && (
        <Row label="Account email">
          <input
            type="text"
            value={site.email ?? ''}
            onChange={(e) => onUpdate({ email: e.target.value || null })}
            placeholder="you@company.com"
            style={inputStyle()}
          />
        </Row>
      )}

      {(site.authKind === 'bearer' || site.authKind === 'basic') && (
        <Row label="Token">
          <div style={{ display: 'flex', gap: 6, alignItems: 'center', flex: 1 }}>
            {site._tokenStored ? (
              <>
                <span style={{ fontFamily: 'var(--mono)', fontSize: 10, color: 'var(--green)' }}>
                  ✓ stored in keychain
                </span>
                <button className="btn" onClick={removeToken}>
                  Remove
                </button>
              </>
            ) : (
              <>
                <input
                  type="password"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  placeholder="Paste API token"
                  style={inputStyle({ flex: 1 })}
                />
                <button className="btn" onClick={saveToken} disabled={!token.trim()}>
                  Save
                </button>
              </>
            )}
          </div>
        </Row>
      )}

      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          paddingTop: 8,
          borderTop: '1px solid var(--b1)',
        }}
      >
        <button
          className="btn"
          onClick={test}
          disabled={testStatus.state === 'busy' || !site.baseUrl}
        >
          {testStatus.state === 'busy' ? 'Testing…' : 'Test connection'}
        </button>
        {testStatus.state === 'ok' && (
          <span style={{ fontSize: 10, color: 'var(--green)', fontFamily: 'var(--mono)' }}>
            ✓ {testStatus.msg}
          </span>
        )}
        {testStatus.state === 'error' && (
          <span style={{ fontSize: 10, color: 'var(--red)', fontFamily: 'var(--mono)' }}>
            ✗ {testStatus.msg}
          </span>
        )}
      </div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
      <label
        style={{
          flex: '0 0 130px',
          fontSize: 9,
          color: 'var(--txt3)',
          textTransform: 'uppercase',
          letterSpacing: '0.07em',
          fontFamily: 'var(--sans)',
          fontWeight: 600,
        }}
      >
        {label}
      </label>
      <div style={{ flex: 1 }}>{children}</div>
    </div>
  );
}

function inputStyle(extra: React.CSSProperties = {}): React.CSSProperties {
  return {
    fontFamily: 'var(--mono)',
    fontSize: 11,
    color: 'var(--txt)',
    background: 'var(--s3)',
    border: '1px solid var(--b1)',
    borderRadius: 4,
    padding: '6px 8px',
    width: '100%',
    ...extra,
  };
}
