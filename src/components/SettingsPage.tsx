import { useEffect, useMemo, useState } from 'react';
import { Settings } from 'lucide-react';
import { useI18n } from '../i18n';
import { backendPlatform } from '../lib/tauri';
import { useLauncherStore } from '../stores/useLauncherStore';
import { GameSettingsPanel } from './settings/GameSettingsPanel';
import { GlobalSettingsPanel } from './settings/GlobalSettingsPanel';
import { SettingsNav, type SettingsTarget } from './settings/SettingsNav';

export function SettingsPage() {
  const { t } = useI18n();
  const [platform, setPlatform] = useState('…');
  const [target, setTarget] = useState<SettingsTarget>({ type: 'global' });
  const games = useLauncherStore((state) => state.games);
  const requestedSettingsGameId = useLauncherStore((state) => state.requestedSettingsGameId);
  const clearRequestedSettingsGame = useLauncherStore((state) => state.clearRequestedSettingsGame);

  useEffect(() => { backendPlatform().then(setPlatform); }, []);
  useEffect(() => {
    if (!requestedSettingsGameId) return;
    if (games.some((game) => game.id === requestedSettingsGameId)) setTarget({ type: 'game', gameId: requestedSettingsGameId });
    clearRequestedSettingsGame();
  }, [requestedSettingsGameId, games, clearRequestedSettingsGame]);

  const selectedGame = useMemo(() => target.type !== 'game' ? undefined : games.find((game) => game.id === target.gameId), [games, target]);
  useEffect(() => { if (target.type === 'game' && !selectedGame) setTarget({ type: 'global' }); }, [selectedGame, target]);

  return (
    <main className="page-shell settings-page-shell">
      <header className="page-header settings-page-header">
        <div><span className="page-eyebrow">{t('settings.eyebrow')}</span><h1>{t('settings.title')}</h1><p>{t('settings.subtitle')}</p></div>
        <div className="page-header-icon"><Settings size={25} /></div>
      </header>
      <div className="settings-layout">
        <SettingsNav games={games} target={target} onChange={setTarget} />
        <div className="settings-workspace">
          {target.type === 'global' || !selectedGame ? <GlobalSettingsPanel platform={platform} /> : <GameSettingsPanel game={selectedGame} />}
        </div>
      </div>
    </main>
  );
}
