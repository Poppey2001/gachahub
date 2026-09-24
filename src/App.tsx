import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Downloads } from './components/Downloads';
import { GamePage } from './components/GamePage';
import { GameRail } from './components/GameRail';
import { Mods } from './components/Mods';
import { Runners } from './components/Runners';
import { SettingsPage } from './components/SettingsPage';
import { backendGames, type XxmiProgress } from './lib/tauri';
import type { InstallJob } from './lib/types';
import { useLauncherStore } from './stores/useLauncherStore';
import { useSettingsStore } from './stores/useSettingsStore';
import { useXxmiStore } from './stores/useXxmiStore';

export default function App() {
  const { section, setGames, upsertJob } = useLauncherStore();
  const globalSettings = useSettingsStore((state) => state.global);
  const refreshXxmi = useXxmiStore((state) => state.refresh);
  const installXxmi = useXxmiStore((state) => state.install);
  const setXxmiProgress = useXxmiStore((state) => state.setProgress);

  useEffect(() => {
    backendGames().then((games) => games.length && setGames(games));
  }, [setGames]);

  useEffect(() => {
    document.documentElement.lang = globalSettings.language;
  }, [globalSettings.language]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    listen<InstallJob>('download-progress', (event) => upsertJob(event.payload))
      .then((unlisten) => { cleanup = unlisten; })
      .catch((error) => console.warn('download event listener unavailable', error));
    return () => cleanup?.();
  }, [upsertJob]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    listen<XxmiProgress>('xxmi-progress', (event) => setXxmiProgress(event.payload))
      .then((unlisten) => { cleanup = unlisten; })
      .catch((error) => console.warn('XXMI event listener unavailable', error));
    return () => cleanup?.();
  }, [setXxmiProgress]);

  useEffect(() => {
    let cancelled = false;
    void refreshXxmi(globalSettings).then((status) => {
      if (
        !cancelled
        && globalSettings.xxmiAutoUpdate
        && status?.installed
        && status.managed
        && status.updateAvailable
      ) {
        void installXxmi(globalSettings);
      }
    });
    return () => { cancelled = true; };
  }, [
    globalSettings.xxmiAutoUpdate,
    globalSettings.xxmiInstallPath,
    globalSettings.xxmiLauncherPath,
    globalSettings.xxmiWineExecutable,
    globalSettings.xxmiWinePrefix,
    refreshXxmi,
    installXxmi,
  ]);

  return (
    <div className="app-shell v3-shell">
      <GameRail />
      <div className="app-workspace">
        {section === 'game' && <GamePage />}
        {section === 'downloads' && <Downloads />}
        {section === 'mods' && <Mods />}
        {section === 'runners' && <Runners />}
        {section === 'settings' && <SettingsPage />}
      </div>
    </div>
  );
}
