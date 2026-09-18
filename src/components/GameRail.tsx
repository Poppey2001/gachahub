import { Download, Gauge, Puzzle, Settings } from 'lucide-react';
import { useLauncherStore, type LauncherSection } from '../stores/useLauncherStore';
import { GameIcon } from './GameArtwork';

const utilityItems: Array<{ id: LauncherSection; icon: typeof Download; label: string }> = [
  { id: 'downloads', icon: Download, label: 'Downloads' },
  { id: 'mods', icon: Puzzle, label: 'Mods' },
  { id: 'runners', icon: Gauge, label: 'Runners' },
];

export function GameRail() {
  const { games, selectedGameId, section, selectGame, setSection } = useLauncherStore();

  return (
    <aside className="game-rail">
      <button className="rail-brand" onClick={() => setSection('game')} aria-label="GachaHub Home">
        <img src="/branding/gachahub-icon.png" alt="GachaHub" />
      </button>

      <div className="rail-divider" />

      <div className="rail-games" aria-label="Spiele">
        {games.map((game) => (
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
      </div>

      <div className="rail-spacer" />
      <div className="rail-tools">
        {utilityItems.map(({ id, icon: Icon, label }) => (
          <button
            key={id}
            className={`rail-tool ${section === id ? 'active' : ''}`}
            onClick={() => setSection(id)}
            title={label}
          >
            <Icon size={20} strokeWidth={1.8} />
          </button>
        ))}
        <div className="rail-divider small" />
        <button
          className={`rail-tool ${section === 'settings' ? 'active' : ''}`}
          onClick={() => setSection('settings')}
          title="Einstellungen"
        >
          <Settings size={20} strokeWidth={1.8} />
        </button>
      </div>
    </aside>
  );
}
