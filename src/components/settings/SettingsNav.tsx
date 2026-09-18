import { Gamepad2, Globe2, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { Game } from '../../lib/types';

export type SettingsTarget =
  | { type: 'global' }
  | { type: 'game'; gameId: string };

interface SettingsNavProps {
  games: Game[];
  target: SettingsTarget;
  onChange: (target: SettingsTarget) => void;
}

export function SettingsNav({ games, target, onChange }: SettingsNavProps) {
  const [query, setQuery] = useState('');
  const filteredGames = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return games;
    return games.filter((game) => game.name.toLowerCase().includes(normalized));
  }, [games, query]);

  return (
    <aside className="settings-nav">
      <button
        className={`settings-nav-item ${target.type === 'global' ? 'active' : ''}`}
        onClick={() => onChange({ type: 'global' })}
      >
        <Globe2 size={17} />
        <span>
          <strong>Global</strong>
          <small>Standard für alle Spiele</small>
        </span>
      </button>

      <div className="settings-nav-heading">Spiele</div>
      <label className="settings-search">
        <Search size={15} />
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Spiel suchen…"
        />
      </label>

      <div className="settings-game-list">
        {filteredGames.map((game) => (
          <button
            key={game.id}
            className={`settings-nav-item ${
              target.type === 'game' && target.gameId === game.id ? 'active' : ''
            }`}
            onClick={() => onChange({ type: 'game', gameId: game.id })}
          >
            <Gamepad2 size={17} style={{ color: game.accent }} />
            <span>
              <strong>{game.name}</strong>
              <small>{game.provider}</small>
            </span>
          </button>
        ))}
      </div>
    </aside>
  );
}
