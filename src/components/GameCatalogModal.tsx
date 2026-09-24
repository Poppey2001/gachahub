import { ArrowDown, ArrowUp, Check, Plus, Search, X } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useI18n } from '../i18n';
import { useLauncherStore } from '../stores/useLauncherStore';
import { GameIcon } from './GameArtwork';

export function GameCatalogModal() {
  const { t } = useI18n();
  const {
    games,
    railGameIds,
    catalogOpen,
    setCatalogOpen,
    addRailGame,
    removeRailGame,
    moveRailGame,
  } = useLauncherStore();
  const [search, setSearch] = useState('');

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    if (!needle) return games;
    return games.filter((game) =>
      [game.name, game.publisher, game.provider, game.shortName]
        .some((value) => value.toLowerCase().includes(needle)),
    );
  }, [games, search]);

  if (!catalogOpen) return null;

  return (
    <div className="catalog-overlay" role="presentation" onMouseDown={() => setCatalogOpen(false)}>
      <section className="game-catalog" role="dialog" aria-modal="true" aria-label={t('catalog.supportedGames')} onMouseDown={(event) => event.stopPropagation()}>
        <header className="catalog-header">
          <div>
            <span className="eyebrow">{t('catalog.eyebrow')}</span>
            <h2>{t('catalog.title')}</h2>
            <p>{t('catalog.description')}</p>
          </div>
          <button className="icon-button" onClick={() => setCatalogOpen(false)} title={t('common.close')}><X size={19} /></button>
        </header>

        <label className="catalog-search">
          <Search size={17} />
          <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder={t('catalog.searchPlaceholder')} autoFocus />
        </label>

        <div className="catalog-list">
          {filtered.map((game) => {
            const added = railGameIds.includes(game.id);
            const railIndex = railGameIds.indexOf(game.id);
            return (
              <article key={game.id} className="catalog-game-card" style={{ '--game-accent': game.accent } as React.CSSProperties}>
                <div className="catalog-game-icon"><GameIcon game={game} /></div>
                <div className="catalog-game-copy">
                  <strong>{game.name}</strong>
                  <span>{game.publisher} · {game.provider}</span>
                  <small>{game.platforms.join(' · ')}</small>
                </div>
                <div className="catalog-game-actions">
                  {added && (
                    <div className="catalog-order-actions">
                      <button className="mini-icon-btn" disabled={railIndex <= 0} onClick={() => moveRailGame(game.id, -1)} title={t('catalog.moveUp')}><ArrowUp size={15} /></button>
                      <button className="mini-icon-btn" disabled={railIndex < 0 || railIndex >= railGameIds.length - 1} onClick={() => moveRailGame(game.id, 1)} title={t('catalog.moveDown')}><ArrowDown size={15} /></button>
                    </div>
                  )}
                  <button
                    className={`catalog-toggle ${added ? 'added' : ''}`}
                    disabled={added && railGameIds.length <= 1}
                    title={added && railGameIds.length <= 1 ? t('catalog.keepOne') : undefined}
                    onClick={() => added ? removeRailGame(game.id) : addRailGame(game.id)}
                  >
                    {added ? <><Check size={15} /> {t('common.added')}</> : <><Plus size={15} /> {t('common.add')}</>}
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      </section>
    </div>
  );
}
