import { useEffect, useState } from 'react';
import { PopoverScreen } from './screens/PopoverScreen';
import { DashboardScreen } from './screens/DashboardScreen';
import { SettingsScreen } from './screens/SettingsScreen';
import { OnboardingScreen } from './screens/OnboardingScreen';
import { useSettings } from './hooks/useSettings';
import { useTheme } from './hooks/useTheme';

type Route = 'popover' | 'dashboard' | 'settings' | 'onboarding';

function readRoute(): Route {
  const hash = window.location.hash.replace(/^#\/?/, '').toLowerCase();
  if (hash === 'dashboard' || hash === 'settings' || hash === 'onboarding') return hash;
  return 'popover';
}

export function App() {
  const [route, setRoute] = useState<Route>(readRoute);
  const { settings } = useSettings();
  useTheme(settings.appearance);

  useEffect(() => {
    const onHash = () => setRoute(readRoute());
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  switch (route) {
    case 'popover':
      return <PopoverScreen />;
    case 'dashboard':
      return <DashboardScreen />;
    case 'settings':
      return <SettingsScreen />;
    case 'onboarding':
      return <OnboardingScreen />;
  }
}
