import { Download, Play, ShieldAlert, Puzzle } from 'lucide-react';
import { useMemo, useState } from 'react';
import { beginInstall } from '../lib/tauri';
import { useLauncherStore } from '../stores/useLauncherStore';

export function GameLibrary() {
  const { games, selectedGameId, selectGame, upsertJob } = useLauncherStore();
  const [busy, setBusy] = useState(false);
  const game = useMemo(() => games.find((g) => g.id === selectedGameId) ?? games[0], [games, selectedGameId]);
  if (!game) return null;

  const install = async () => {
    setBusy(true);
    try {
      const job = await beginInstall(game.id);
      upsertJob(job);
    } catch {
      upsertJob({
        id: `demo-${game.id}`, gameId: game.id, phase: 'downloading', progress: 0.12,
        downloadedBytes: 1_288_490_188, totalBytes: 10_737_418_240, speedBytes: 43_200_000,
      });
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="content">
      <header className="topbar">
        <div><h1>Bibliothek</h1><p>Alle Spiele, Provider und Kompatibilitätsprofile an einem Ort.</p></div>
        <div className="chip">{games.length} Spiele</div>
      </header>

      <section className="library-grid">
        <div className="game-list">
          {games.map((g) => (
            <button key={g.id} onClick={() => selectGame(g.id)} className={g.id === game.id ? 'game-row active' : 'game-row'}>
              <span className="game-dot" style={{ background: g.accent }}/>
              <span><strong>{g.name}</strong><small>{g.provider}</small></span>
              <em>{g.status === 'ready' ? 'Bereit' : 'Installieren'}</em>
            </button>
          ))}
        </div>

        <div className="game-detail" style={{'--accent': game.accent} as React.CSSProperties}>
          <div className="hero-glow" />
          <div className="provider-label">{game.provider.toUpperCase()} PROVIDER</div>
          <h2>{game.name}</h2>
          <p>{game.publisher}</p>
          <div className="meta-grid">
            <div><span>Plattformen</span><strong>{game.platforms.join(' • ')}</strong></div>
            <div><span>Installation</span><strong>{game.installModes.join(' • ')}</strong></div>
            <div><span>Mod-Richtlinie</span><strong className={game.modPolicy === 'restricted' ? 'warning' : ''}>{game.modPolicy}</strong></div>
          </div>
          {game.notes && <div className="notice"><ShieldAlert size={18}/>{game.notes}</div>}
          <div className="actions">
            <button className="primary" onClick={install} disabled={busy}><Download size={18}/>{busy ? 'Vorbereiten…' : 'Installieren'}</button>
            <button><Play size={18}/> Importieren</button>
            <button><Puzzle size={18}/> Mods</button>
          </div>
        </div>
      </section>
    </main>
  );
}
