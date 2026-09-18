import { useEffect } from 'react';
import { Downloads } from './components/Downloads';
import { GamePage } from './components/GamePage';
import { GameRail } from './components/GameRail';
import { Mods } from './components/Mods';
import { Runners } from './components/Runners';
import { SettingsPage } from './components/SettingsPage';
import { backendGames } from './lib/tauri';
import { useLauncherStore } from './stores/useLauncherStore';

export default function App() {
  const { section, setGames } = useLauncherStore();

  useEffect(() => {
    backendGames().then((games) => games.length && setGames(games));
  }, [setGames]);

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
