import { useEffect, useState } from 'react';
import type { ToastEvent } from '@pulse/types';
import { isTauri, onToast } from '../lib/tauri';

interface ActiveToast extends ToastEvent {
  id: number;
}

export function ToastHost() {
  const [toasts, setToasts] = useState<ActiveToast[]>([]);

  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    let counter = 0;
    (async () => {
      unlisten = await onToast((t) => {
        const id = ++counter;
        setToasts((cur) => [...cur, { ...t, id }]);
        setTimeout(() => setToasts((cur) => cur.filter((x) => x.id !== id)), 5000);
      });
    })();
    return () => unlisten?.();
  }, []);

  if (toasts.length === 0) return null;
  return (
    <div className="toast-host">
      {toasts.map((t) => (
        <div key={t.id} className={`toast ${t.kind}`}>
          <div className="toast-title">{t.title}</div>
          <div className="toast-body">{t.body}</div>
        </div>
      ))}
    </div>
  );
}
