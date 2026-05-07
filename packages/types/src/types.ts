// Mirrors `crates/core/src/model.rs` field-for-field. Both sides camelCase.

export type AttributionConfidence = 'high' | 'medium' | 'low';
export type SessionState = 'normal' | 'warn' | 'crit' | 'idle';

export interface UsageTotals {
  inputTokens: number;
  outputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  totalTokens: number;
  costUsd: number;
  calls: number;
  cacheHitRate: number;
}

export interface TaskSnapshot {
  taskId: string | null;
  taskName: string | null;
  branch: string | null;
  cwd: string | null;
  model: string | null;
  confidence: AttributionConfidence;
  confidenceScore: number;
  usage: UsageTotals;
  firstSeen: string;
  lastSeen: string;
  metadata: TaskMetadata | null;
}

export interface TaskMetadata {
  taskId: string;
  enricher: string;
  title: string | null;
  status: string | null;
  assignee: string | null;
  url: string | null;
  projectKey: string | null;
  issueType: string | null;
  priority: string | null;
  fetchedAt: string;
}

export type EnrichmentState = 'idle' | 'running' | 'disabled' | 'error';

export interface EnrichmentStatus {
  state: EnrichmentState;
  lastRunAt: string | null;
  lastError: string | null;
  pendingCount: number;
  enrichers: string[];
}

export type JiraAuthKind = 'none' | 'bearer' | 'basic' | 'oauth-2';

export type OAuthProviderId = 'atlassian' | 'github' | 'custom';

export interface OAuthBeginInput {
  provider: OAuthProviderId;
  siteId: string;
  clientId: string;
  scopes?: string[];
}

export interface OAuthBeginOutput {
  authorizeUrl: string;
  state: string;
}

export interface OAuthCompleteInput {
  provider: OAuthProviderId;
  state: string;
  code: string;
}

export type PluginCategory =
  | 'issue-tracking'
  | 'source-control'
  | 'communication'
  | 'documentation'
  | 'ai-provider'
  | 'observability';

export type PluginCapability =
  | 'enrich-task'
  | 'attribute-turn'
  | 'ingest-transcript'
  | 'send-notification';

export type AuthMethodKind =
  | 'none'
  | 'pat'
  | 'basic-email-token'
  | 'oauth-2-pkce'
  | 'github-app';

export type PluginConnectStyle = 'single-instance' | 'multi-instance';

export interface PluginManifest {
  id: string;
  displayName: string;
  vendor: string;
  description: string;
  category: PluginCategory;
  capabilities: PluginCapability[];
  authMethods: AuthMethodKind[];
  preferredAuth: AuthMethodKind;
  connectStyle: PluginConnectStyle;
  icon: string;
  docsUrl: string | null;
}

export type PluginState = 'not-connected' | 'connecting' | 'connected' | 'error' | 'disabled';
export type InstanceState = 'needs-credentials' | 'connected' | 'error' | 'disabled';

export interface InstanceStatus {
  instanceId: string;
  state: InstanceState;
  lastCheck: string | null;
  error: string | null;
}

export interface PluginStatus {
  pluginId: string;
  state: PluginState;
  instances: InstanceStatus[];
  lastCheck: string | null;
  error: string | null;
}

export interface PluginInstanceSummary {
  instanceId: string;
  label: string;
  subtitle: string | null;
  enabled: boolean;
}

export interface JiraSite {
  id: string;
  label: string;
  baseUrl: string;
  projectKeys: string[];
  authKind: JiraAuthKind;
  email: string | null;
  oauthClientId: string | null;
  enabled: boolean;
}

export interface JiraConfig {
  sites: JiraSite[];
}

export interface ActiveTask {
  task: TaskSnapshot | null;
  sessionUsedPct: number;
  sessionResetAt: string | null;
  weeklyUsedPct: number;
  weeklyResetAt: string | null;
  state: SessionState;
}

export interface DashboardSummary {
  range: { from: string; to: string };
  totals: UsageTotals;
  tasks: TaskSnapshot[];
  unattributed: UsageTotals;
  daily: Array<{ date: string; tokens: number; cost: number; calls: number }>;
  modelShare: Array<{ model: string; tokens: number; pct: number }>;
}

export interface Settings {
  branchRegex: string;
  pollIntervalMs: number;
  weeklyTokenBudget: number;
  sessionTokenBudget: number;
  warnThresholdPct: number;
  critThresholdPct: number;
  notifyOnWarn: boolean;
  notifyOnCrit: boolean;
  notifyDailySummary: boolean;
  appearance: 'dark' | 'light' | 'auto';
  startAtLogin: boolean;
  enrichmentEnabled: boolean;
  enrichmentIntervalSecs: number;
  enrichmentCacheTtlSecs: number;
  jira: JiraConfig;
}

export interface OnboardingStatus {
  claudeDirFound: boolean;
  claudeDirPath: string | null;
  sessionsDiscovered: number;
  ingestComplete: boolean;
}

export const PulseEvent = {
  ActiveTaskChanged: 'pulse://active-task-changed',
  UsageUpdated: 'pulse://usage-updated',
  ThresholdCrossed: 'pulse://threshold-crossed',
  IngestProgress: 'pulse://ingest-progress',
  Toast: 'pulse://toast',
  TaskEnriched: 'pulse://task-enriched',
  EnrichmentStatus: 'pulse://enrichment-status',
  EnrichmentError: 'pulse://enrichment-error',
} as const;
export type PulseEvent = (typeof PulseEvent)[keyof typeof PulseEvent];

export interface ThresholdEvent {
  kind: 'session-warn' | 'session-crit' | 'weekly-warn' | 'weekly-crit';
  pct: number;
  taskId: string | null;
}

export interface ToastEvent {
  kind: 'success' | 'info' | 'warn' | 'crit';
  title: string;
  body: string;
}
