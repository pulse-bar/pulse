import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ActiveTask,
  DashboardSummary,
  OnboardingStatus,
  Settings,
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
