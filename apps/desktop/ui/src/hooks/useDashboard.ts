import { useCallback, useEffect, useState } from 'react';
import type { DashboardSummary } from '@pulse/types';
import { getDashboard, isTauri, onUsageUpdated } from '../lib/tauri';

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
    let unlisten: (() => void) | null = null;
    (async () => {
      unlisten = await onUsageUpdated(() => refresh());
    })();
    return () => unlisten?.();
  }, [refresh]);

  return { data, loading, refresh };
}
