import { Globe2, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useI18n } from '../../i18n';
import type { Game } from '../../lib/types';
import { GameIcon } from '../GameArtwork';

export type SettingsTarget = { type: 'global' } | { type: 'game'; gameId: string };

interface SettingsNavProps {
  games: Game[];
  target: SettingsTarget;
  onChange: (target: SettingsTarget) => void;
}

export function SettingsNav({ games, target, onChange }: SettingsNavProps) {
  const { t } = useI18n();
  const [query, setQuery] = useState('');
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return games;
    return games.filter((game) => game.name.toLowerCase().includes(needle));
  }, [games, query]);

  return (
    <aside className="settings-nav">
      <button
        className={`settings-nav-item global ${target.type === 'global' ? 'active' : ''}`}
        onClick={() => onChange({ type: 'global' })}
      >
        <span className="settings-nav-icon"><Globe2 size={18} /></span>
        <span><strong>{t('settings.global')}</strong><small>{t('settings.globalHint')}</small></span>
      </button>
      <div className="settings-nav-heading">{t('settings.games')}</div>
      <label className="settings-nav-search">
        <Search size={15} />
        <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder={t('settings.searchGame')} />
      </label>
      <div className="settings-nav-games">
        {filtered.map((game) => (
          <button
            key={game.id}
            className={`settings-nav-item ${target.type === 'game' && target.gameId === game.id ? 'active' : ''}`}
            onClick={() => onChange({ type: 'game', gameId: game.id })}
          >
            <span className="settings-nav-icon game"><GameIcon game={game} /></span>
            <span><strong>{game.name}</strong><small>{game.provider}</small></span>
          </button>
        ))}
      </div>
    </aside>
  );
}
