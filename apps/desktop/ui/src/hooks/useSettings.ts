import { useCallback, useEffect, useState } from 'react';
import { emit, listen } from '@tauri-apps/api/event';
import type { Settings } from '@pulse/types';
import { getSettings, isTauri, saveSettings } from '../lib/tauri';

const DEFAULTS: Settings = {
  branchRegex: '(?i)([A-Z][A-Z0-9]+-\\d+)',
  pollIntervalMs: 250,
  weeklyTokenBudget: 5_000_000,
  sessionTokenBudget: 200_000,
  warnThresholdPct: 0.78,
  critThresholdPct: 0.92,
  notifyOnWarn: true,
  notifyOnCrit: true,
  notifyDailySummary: true,
  appearance: 'dark',
  startAtLogin: true,
  enrichmentEnabled: true,
  enrichmentIntervalSecs: 30,
  enrichmentCacheTtlSecs: 6 * 60 * 60,
  jira: { sites: [] },
};

const SETTINGS_CHANGED = 'pulse://settings-changed';

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    let alive = true;
    let unlisten: (() => void) | null = null;

    (async () => {
      try {
        const fetched = await getSettings();
        if (alive) setSettings(fetched);
      } catch (err) {
        console.warn('getSettings failed', err);
      } finally {
        if (alive) setLoading(false);
      }

      try {
        unlisten = await listen<Settings>(SETTINGS_CHANGED, (e) => {
          if (alive) setSettings(e.payload);
        });
      } catch (err) {
        console.warn('subscribe settings-changed failed', err);
      }
    })();

    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  const save = useCallback(async (next: Settings) => {
    setSettings(next);
    if (isTauri()) {
      await saveSettings(next);
      await emit(SETTINGS_CHANGED, next);
    }
  }, []);

  return { settings, loading, save };
}
