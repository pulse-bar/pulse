import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ActiveTask,
  DashboardSummary,
  EnrichmentStatus,
  JiraSite,
  OnboardingStatus,
  PluginInstanceSummary,
  PluginManifest,
  PluginStatus,
  Settings,
  TaskMetadata,
  ThresholdEvent,
  ToastEvent,
} from '@pulse/types';

export const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function getActiveTask(): Promise<ActiveTask> {
  return tauriInvoke<ActiveTask>('get_active_task');
}

export async function getDashboard(days = 7): Promise<DashboardSummary> {
  return tauriInvoke<DashboardSummary>('get_dashboard', { days });
}

export async function getSettings(): Promise<Settings> {
  return tauriInvoke<Settings>('get_settings');
}

export async function saveSettings(settings: Settings): Promise<void> {
  await tauriInvoke('save_settings', { settings });
}

export async function getOnboardingStatus(): Promise<OnboardingStatus> {
  return tauriInvoke<OnboardingStatus>('get_onboarding_status');
}

export async function openDashboard(): Promise<void> {
  await tauriInvoke('open_dashboard');
}

export async function openSettingsWindow(): Promise<void> {
  await tauriInvoke('open_settings');
}

export async function resetDatabase(): Promise<void> {
  await tauriInvoke('reset_database');
}

export async function triggerFullRescan(): Promise<number> {
  return tauriInvoke<number>('trigger_full_rescan');
}

export async function getTaskMetadata(taskId: string): Promise<TaskMetadata | null> {
  return tauriInvoke<TaskMetadata | null>('get_task_metadata', { taskId });
}

export async function getEnrichmentStatus(): Promise<EnrichmentStatus> {
  return tauriInvoke<EnrichmentStatus>('get_enrichment_status');
}

export async function runEnrichmentNow(): Promise<number> {
  return tauriInvoke<number>('run_enrichment_now');
}

export async function saveJiraSites(sites: JiraSite[]): Promise<void> {
  await tauriInvoke('save_jira_sites', { sites });
}

export async function upsertJiraSite(site: JiraSite): Promise<void> {
  await tauriInvoke('upsert_jira_site', { site });
}

export async function deleteJiraSite(siteId: string): Promise<void> {
  await tauriInvoke('delete_jira_site', { siteId });
}

export async function storeJiraToken(
  siteId: string,
  authKind: 'bearer' | 'basic',
  token: string,
): Promise<void> {
  await tauriInvoke('store_jira_token', { siteId, authKind, token });
}

export async function oauthBegin(
  input: import('@pulse/types').OAuthBeginInput,
): Promise<import('@pulse/types').OAuthBeginOutput> {
  return tauriInvoke('oauth_begin', { input });
}

export async function oauthComplete(
  input: import('@pulse/types').OAuthCompleteInput,
): Promise<void> {
  await tauriInvoke('oauth_complete', { input });
}

export async function listPlugins(): Promise<PluginManifest[]> {
  return tauriInvoke<PluginManifest[]>('list_plugins');
}

export async function listPluginStatuses(): Promise<PluginStatus[]> {
  return tauriInvoke<PluginStatus[]>('list_plugin_statuses');
}

export async function listPluginInstances(pluginId: string): Promise<PluginInstanceSummary[]> {
  return tauriInvoke<PluginInstanceSummary[]>('list_plugin_instances', { pluginId });
}

export async function testPluginInstance(pluginId: string, instanceId: string): Promise<void> {
  await tauriInvoke('test_plugin_instance', { pluginId, instanceId });
}

export async function jiraTokenPresent(siteId: string): Promise<boolean> {
  return tauriInvoke<boolean>('jira_token_present', { siteId });
}

export async function deleteJiraToken(siteId: string): Promise<void> {
  await tauriInvoke('delete_jira_token', { siteId });
}

export async function testJiraSite(siteId: string): Promise<void> {
  await tauriInvoke('test_jira_site', { siteId });
}

export interface ConnectJiraOutput {
  authorizeUrl: string;
  redirectUri: string;
  port: number;
}

export interface OAuthResult {
  siteId: string;
  ok: boolean;
  error?: string | null;
}

export async function connectJiraOauth(siteId: string, clientId: string): Promise<ConnectJiraOutput> {
  return tauriInvoke<ConnectJiraOutput>('connect_jira_oauth', {
    input: { siteId, clientId },
  });
}

export async function onOauthResult(cb: (r: OAuthResult) => void): Promise<UnlistenFn> {
  return tauriListen<OAuthResult>('pulse://oauth-result', (e) => cb(e.payload));
}

export async function onTaskEnriched(cb: (m: TaskMetadata) => void): Promise<UnlistenFn> {
  return tauriListen<TaskMetadata>('pulse://task-enriched', (e) => cb(e.payload));
}

export async function onEnrichmentStatus(cb: (s: EnrichmentStatus) => void): Promise<UnlistenFn> {
  return tauriListen<EnrichmentStatus>('pulse://enrichment-status', (e) => cb(e.payload));
}

export async function onActiveTaskChanged(cb: (a: ActiveTask) => void): Promise<UnlistenFn> {
  return tauriListen<ActiveTask>('pulse://active-task-changed', (e) => cb(e.payload));
}

export async function onUsageUpdated(cb: () => void): Promise<UnlistenFn> {
  return tauriListen('pulse://usage-updated', () => cb());
}

export async function onThresholdCrossed(cb: (t: ThresholdEvent) => void): Promise<UnlistenFn> {
  return tauriListen<ThresholdEvent>('pulse://threshold-crossed', (e) => cb(e.payload));
}

export async function onToast(cb: (t: ToastEvent) => void): Promise<UnlistenFn> {
  return tauriListen<ToastEvent>('pulse://toast', (e) => cb(e.payload));
}
