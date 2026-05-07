import type { PluginManifest, PluginState, PluginStatus } from '@pulse/types';

const STATE_LABEL: Record<PluginState, string> = {
  'not-connected': 'Not connected',
  connecting: 'Connecting…',
  connected: 'Connected',
  error: 'Error',
  disabled: 'Disabled',
};

const STATE_COLOR: Record<PluginState, string> = {
  'not-connected': 'var(--txt3)',
  connecting: 'var(--cyan)',
  connected: 'var(--green)',
  error: 'var(--red)',
  disabled: 'var(--txt3)',
};

export function IntegrationCard({
  manifest,
  status,
  onClick,
}: {
  manifest: PluginManifest;
  status: PluginStatus | undefined;
  onClick: () => void;
}) {
  const state = status?.state ?? 'not-connected';
  const instanceCount = status?.instances.length ?? 0;
  const error = status?.error;

  return (
    <button
      onClick={onClick}
      style={{
        textAlign: 'left',
        background: 'var(--s2)',
        border: '1px solid var(--b1)',
        borderRadius: 8,
        padding: 14,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        cursor: 'pointer',
        transition: 'border-color 0.15s, background 0.15s',
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = 'var(--b2)';
        e.currentTarget.style.background = 'var(--s3)';
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = 'var(--b1)';
        e.currentTarget.style.background = 'var(--s2)';
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <span
          style={{
            width: 32,
            height: 32,
            borderRadius: 6,
            background: 'var(--s3)',
            border: '1px solid var(--b1)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontFamily: 'var(--sans)',
            fontWeight: 700,
            fontSize: 13,
            color: 'var(--pulse)',
          }}
        >
          {manifest.displayName.slice(0, 1)}
        </span>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontFamily: 'var(--sans)',
              fontSize: 12,
              fontWeight: 600,
              color: 'var(--txt)',
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
            {manifest.vendor} · {humanCategory(manifest.category)}
          </div>
        </div>
      </div>

      <div
        style={{
          fontFamily: 'var(--sans)',
          fontSize: 11,
          color: 'var(--txt2)',
          lineHeight: 1.45,
        }}
      >
        {manifest.description}
      </div>

      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          paddingTop: 8,
          borderTop: '1px solid var(--b1)',
        }}
      >
        <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
          <span
            style={{
              width: 7,
              height: 7,
              borderRadius: '50%',
              background: STATE_COLOR[state],
              boxShadow: state === 'connected' ? `0 0 6px ${STATE_COLOR[state]}` : 'none',
            }}
          />
          <span style={{ fontFamily: 'var(--mono)', fontSize: 9, color: 'var(--txt2)' }}>
            {STATE_LABEL[state]}
            {instanceCount > 0 && ` · ${instanceCount}`}
          </span>
        </span>
        <span style={{ fontFamily: 'var(--mono)', fontSize: 9, color: 'var(--txt3)' }}>
          Configure →
        </span>
      </div>

      {error && (
        <div
          style={{
            fontFamily: 'var(--mono)',
            fontSize: 9,
            color: 'var(--red)',
            background: 'var(--red-d)',
            padding: '6px 8px',
            borderRadius: 4,
            border: '1px solid rgba(220, 70, 70, 0.25)',
            wordBreak: 'break-word',
          }}
        >
          {error}
        </div>
      )}
    </button>
  );
}

function humanCategory(c: PluginManifest['category']): string {
  switch (c) {
    case 'issue-tracking':
      return 'Issue tracking';
    case 'source-control':
      return 'Source control';
    case 'communication':
      return 'Communication';
    case 'documentation':
      return 'Documentation';
    case 'ai-provider':
      return 'AI provider';
    case 'observability':
      return 'Observability';
  }
}
