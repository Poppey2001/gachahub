import { Download, Gauge, Plus, Puzzle, Settings } from 'lucide-react';
import { useI18n } from '../i18n';
import { useLauncherStore, type LauncherSection } from '../stores/useLauncherStore';
import { GameIcon } from './GameArtwork';
import { GameCatalogModal } from './GameCatalogModal';

const utilityItems: Array<{ id: LauncherSection; icon: typeof Download; labelKey: string }> = [
  { id: 'downloads', icon: Download, labelKey: 'common.downloads' },
  { id: 'mods', icon: Puzzle, labelKey: 'common.mods' },
  { id: 'runners', icon: Gauge, labelKey: 'common.runners' },
];

export function GameRail() {
  const { t } = useI18n();
  const {
    games,
    railGameIds,
    selectedGameId,
    section,
    selectGame,
    setSection,
    setCatalogOpen,
  } = useLauncherStore();

  const railGames = railGameIds
    .map((id) => games.find((game) => game.id === id))
    .filter((game): game is NonNullable<typeof game> => Boolean(game));

  return (
    <>
      <aside className="game-rail">
        <button className="rail-brand" onClick={() => setSection('game')} aria-label={t('nav.home')}>
          <img src="/branding/gachahub-icon.png" alt="GachaHub" />
        </button>

        <div className="rail-divider" />

        <div className="rail-games" aria-label={t('nav.myGames')}>
          {railGames.map((game) => (
            <button
              key={game.id}
              className={`rail-game ${section === 'game' && selectedGameId === game.id ? 'active' : ''}`}
              onClick={() => selectGame(game.id)}
              title={game.name}
              style={{ '--game-accent': game.accent } as React.CSSProperties}
            >
              <GameIcon game={game} />
            </button>
          ))}
          <button className="rail-game rail-add-game" onClick={() => setCatalogOpen(true)} title={t('nav.addSupportedGame')}>
            <Plus size={21} />
          </button>
        </div>

        <div className="rail-spacer" />
        <div className="rail-tools">
          {utilityItems.map(({ id, icon: Icon, labelKey }) => (
            <button
              key={id}
              className={`rail-tool ${section === id ? 'active' : ''}`}
              onClick={() => setSection(id)}
              title={t(labelKey)}
            >
              <Icon size={20} strokeWidth={1.8} />
            </button>
          ))}
          <div className="rail-divider small" />
          <button
            className={`rail-tool ${section === 'settings' ? 'active' : ''}`}
            onClick={() => setSection('settings')}
            title={t('common.settings')}
          >
            <Settings size={20} strokeWidth={1.8} />
          </button>
        </div>
      </aside>
      <GameCatalogModal />
    </>
  );
}
