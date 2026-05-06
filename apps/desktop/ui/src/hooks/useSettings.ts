import { useCallback, useEffect, useState } from 'react';
import type { Settings } from '@pulse/types';
import { getSettings, isTauri, saveSettings } from '../lib/tauri';

const DEFAULTS: Settings = {
  jiraBaseUrl: null,
  jiraProjectKeys: [],
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
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    (async () => {
      try {
        setSettings(await getSettings());
      } catch (err) {
        console.warn('getSettings failed', err);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const save = useCallback(async (next: Settings) => {
    setSettings(next);
    if (isTauri()) await saveSettings(next);
  }, []);

  return { settings, loading, save };
}
