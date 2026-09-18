import { Download, FolderOpen, MoreHorizontal, Play, Puzzle, Settings2, ShieldAlert, Sparkles } from 'lucide-react';
import { useMemo, useState } from 'react';
import { beginInstall } from '../lib/tauri';
import { resolveGameSettings } from '../lib/settings';
import { useLauncherStore } from '../stores/useLauncherStore';
import { useSettingsStore } from '../stores/useSettingsStore';
import { GameBackdrop, GameLogo } from './GameArtwork';
import { getCompatibilityPreset } from '../data/compatibilityPresets';

function fmt(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i >= 3 ? 1 : 0)} ${units[i]}`;
}

export function GamePage() {
  const { games, selectedGameId, jobs, upsertJob, openGameSettings, setSection } = useLauncherStore();
  const [busy, setBusy] = useState(false);
  const globalSettings = useSettingsStore((state) => state.global);
  const gameSettings = useSettingsStore((state) => state.games[selectedGameId]);
  const patchGame = useSettingsStore((state) => state.patchGame);
  const game = useMemo(() => games.find((entry) => entry.id === selectedGameId) ?? games[0], [games, selectedGameId]);
  const job = useMemo(() => jobs.find((entry) => entry.gameId === game?.id), [jobs, game?.id]);

  if (!game) return null;
  const effectiveSettings = resolveGameSettings(game.id, globalSettings, gameSettings);
  const compatibility = getCompatibilityPreset(game.id);

  const install = async () => {
    setBusy(true);
    try {
      const result = await beginInstall(game.id);
      upsertJob(result);
    } catch {
      upsertJob({
        id: `demo-${game.id}`,
        gameId: game.id,
        phase: 'downloading',
        progress: 0.34,
        downloadedBytes: 10_737_418_240,
        totalBytes: 31_212_879_872,
        speedBytes: 61_900_000,
      });
    } finally {
      setBusy(false);
    }
  };

  const progress = Math.round((job?.progress ?? 0) * 100);

  return (
    <main
      className="game-stage"
      style={{ '--accent': game.accent, '--accent-2': game.accent2 } as React.CSSProperties}
    >
      <GameBackdrop game={game} />
      <div className="stage-grid" />
      <div className="stage-watermark" aria-hidden="true">{game.shortName}</div>
      <div className="stage-vignette" />

      <header className="stage-topbar">
        <div>
          <img className="topbar-brand-icon" src="/branding/gachahub-icon.png" alt="" />
          <span className="stage-product">GachaHub</span>
          <span className="stage-version">v0.7</span>
        </div>
        <div className="topbar-actions">
          <span className="network-pill"><i /> Online</span>
          <button className="icon-button" onClick={() => openGameSettings(game.id)} title="Spiel-Einstellungen">
            <Settings2 size={18} />
          </button>
          <button className="icon-button" title="Mehr"><MoreHorizontal size={19} /></button>
        </div>
      </header>

      <section className="hero-content">
        <GameLogo game={game} />
        <div className="provider-kicker">{game.provider} / {game.version ?? 'stable'}</div>
        <h1>{game.name}</h1>
        <p className="hero-subtitle">{game.subtitle}</p>

        <div className="hero-meta">
          <span>{game.publisher}</span>
          <b>•</b>
          <span>{game.platforms[0]}</span>
          <b>•</b>
          <span className={game.modPolicy === 'restricted' ? 'restricted-text' : ''}>{game.modPolicy} mods</span>
          {compatibility && (<>
            <b>•</b>
            <span className={`compat-hero-state ${compatibility.linux.state}`}>Linux: {compatibility.linux.label}</span>
          </>)}
          {game.xxmiImporter && (<>
            <b>•</b>
            <span className={effectiveSettings.xxmiEnabled ? 'xxmi-hero-state enabled' : 'xxmi-hero-state'}>XXMI {game.xxmiImporter} {effectiveSettings.xxmiEnabled ? 'ON' : 'OFF'}</span>
          </>)}
        </div>

        {game.notes && (
          <div className="hero-notice"><ShieldAlert size={16} /> {game.notes}</div>
        )}

        {job && job.phase !== 'ready' ? (
          <div className="inline-download">
            <div className="inline-download-head">
              <div>
                <span>{job.phase}</span>
                <strong>{progress}%</strong>
              </div>
              <small>{fmt(job.downloadedBytes)} / {fmt(job.totalBytes)} · {fmt(job.speedBytes)}/s</small>
            </div>
            <div className="hero-progress"><i style={{ width: `${progress}%` }} /></div>
          </div>
        ) : (
          <div className="hero-actions">
            <button className="play-button" onClick={install} disabled={busy}>
              {game.status === 'ready' ? <Play size={20} fill="currentColor" /> : <Download size={20} />}
              {busy ? 'Vorbereiten…' : game.status === 'ready' ? 'Spielen' : 'Installieren'}
            </button>
            <button className="ghost-button"><FolderOpen size={18} /> Importieren</button>
            {game.modPolicy !== 'disabled' && <button className="ghost-button" onClick={() => setSection('mods')}><Puzzle size={18} /> Mods</button>}
            {game.xxmiImporter && (
              <button
                className={`ghost-button xxmi-quick-toggle ${effectiveSettings.xxmiEnabled ? 'active' : ''}`}
                onClick={() => patchGame(game.id, { xxmiEnabled: !effectiveSettings.xxmiEnabled })}
                title={`${game.xxmiImporter} ${effectiveSettings.xxmiEnabled ? 'deaktivieren' : 'aktivieren'}`}
              >
                <Sparkles size={18} /> XXMI {effectiveSettings.xxmiEnabled ? 'ON' : 'OFF'}
              </button>
            )}
          </div>
        )}
      </section>

      <section className="stage-footer">
        <div className="stage-stat"><span>Status</span><strong>{job ? job.phase : game.status === 'ready' ? 'Bereit' : 'Nicht installiert'}</strong></div>
        <div className="stage-stat"><span>Installationsart</span><strong>{game.installModes[0]}</strong></div>
        <div className="stage-stat"><span>Runtime</span><strong>{effectiveSettings.runtime} · {effectiveSettings.runnerVersion || 'default'}</strong></div>
        <div className="stage-stat"><span>Spielzeit</span><strong>—</strong></div>
      </section>
    </main>
  );
}
