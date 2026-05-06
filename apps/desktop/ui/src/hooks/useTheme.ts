import { useEffect } from 'react';
import type { Settings } from '@pulse/types';

type Resolved = 'dark' | 'light';

function apply(theme: Resolved) {
  document.documentElement.setAttribute('data-theme', theme);
}

// Applies the theme to <html data-theme="..."> based on settings.
// "auto" follows the OS preference and updates live as it changes.
export function useTheme(appearance: Settings['appearance']) {
  useEffect(() => {
    if (appearance !== 'auto') {
      apply(appearance);
      return;
    }
    const media = window.matchMedia('(prefers-color-scheme: light)');
    const sync = () => apply(media.matches ? 'light' : 'dark');
    sync();
    media.addEventListener('change', sync);
    return () => media.removeEventListener('change', sync);
  }, [appearance]);
}
