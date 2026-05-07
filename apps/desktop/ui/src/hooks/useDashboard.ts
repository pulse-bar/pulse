import { useCallback, useEffect, useState } from 'react';
import type { DashboardSummary } from '@pulse/types';
import { getDashboard, isTauri, onTaskEnriched, onUsageUpdated } from '../lib/tauri';

export function useDashboard(days: number) {
  const [data, setData] = useState<DashboardSummary | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      setData(await getDashboard(days));
    } catch (err) {
      console.warn('getDashboard failed', err);
    } finally {
      setLoading(false);
    }
  }, [days]);

  useEffect(() => {
    refresh();
    if (!isTauri()) return;
    const unlisteners: Array<() => void> = [];
    (async () => {
      unlisteners.push(await onUsageUpdated(() => refresh()));
      unlisteners.push(await onTaskEnriched(() => refresh()));
    })();
    return () => unlisteners.forEach((u) => u());
  }, [refresh]);

  return { data, loading, refresh };
}
