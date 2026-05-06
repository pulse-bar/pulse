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
  jiraBaseUrl: string | null;
  jiraProjectKeys: string[];
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
