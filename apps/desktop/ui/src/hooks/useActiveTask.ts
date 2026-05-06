import { useEffect, useState } from 'react';
import type { ActiveTask } from '@pulse/types';
import { getActiveTask, isTauri, onActiveTaskChanged } from '../lib/tauri';

const FALLBACK: ActiveTask = {
  task: null,
  sessionUsedPct: 0,
  sessionResetAt: null,
  weeklyUsedPct: 0,
  weeklyResetAt: null,
  state: 'idle',
};

export function useActiveTask(): ActiveTask {
  const [active, setActive] = useState<ActiveTask>(FALLBACK);

  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    let unlisten: (() => void) | null = null;

    (async () => {
      try {
        const a = await getActiveTask();
        if (alive) setActive(a);
      } catch (err) {
        console.warn('getActiveTask failed', err);
      }
      try {
        unlisten = await onActiveTaskChanged((a) => {
          if (alive) setActive(a);
        });
      } catch (err) {
        console.warn('subscribe activeTask failed', err);
      }
    })();

    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  return active;
}
