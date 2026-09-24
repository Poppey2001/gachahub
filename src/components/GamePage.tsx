import { Download, FolderOpen, MoreHorizontal, Play, Puzzle, Settings2, ShieldAlert, Sparkles } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useI18n } from '../i18n';
import { beginInstall, chooseDirectory, launchGame, postInstallDefaults } from '../lib/tauri';
import { resolveGameSettings } from '../lib/settings';
import { useLauncherStore } from '../stores/useLauncherStore';
import { useSettingsStore } from '../stores/useSettingsStore';
import { useXxmiStore } from '../stores/useXxmiStore';
import { GameBackdrop, GameLogo } from './GameArtwork';
import { LanguageSwitcher } from './LanguageSwitcher';
import { getCompatibilityPreset } from '../data/compatibilityPresets';

function fmt(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i >= 3 ? 1 : 0)} ${units[i]}`;
}

export function GamePage() {
  const { t } = useI18n();
  const { games, selectedGameId, jobs, upsertJob, openGameSettings, setSection } = useLauncherStore();
  const [busy, setBusy] = useState(false);
  const [launchBusy, setLaunchBusy] = useState(false);
  const [installError, setInstallError] = useState('');
  const [xxmiActionBusy, setXxmiActionBusy] = useState(false);
  const [pathBusy, setPathBusy] = useState(false);
  const xxmiStatus = useXxmiStore((state) => state.status);
  const installXxmi = useXxmiStore((state) => state.install);
  const launchXxmi = useXxmiStore((state) => state.launch);
  const refreshXxmi = useXxmiStore((state) => state.refresh);
  const globalSettings = useSettingsStore((state) => state.global);
  const gameSettings = useSettingsStore((state) => state.games[selectedGameId]);
  const patchGame = useSettingsStore((state) => state.patchGame);
  const applyDetectedGameDefaults = useSettingsStore((state) => state.applyDetectedGameDefaults);
  const setGameOverride = useSettingsStore((state) => state.setGameOverride);
  const game = useMemo(() => games.find((entry) => entry.id === selectedGameId) ?? games[0], [games, selectedGameId]);
  const job = useMemo(() => jobs.find((entry) => entry.gameId === game?.id), [jobs, game?.id]);

  if (!game) return null;
  const effectiveSettings = resolveGameSettings(game.id, globalSettings, gameSettings);
  const compatibility = getCompatibilityPreset(game.id);

  const chooseInstallFolder = async () => {
    if (pathBusy) return;
    setPathBusy(true);
    setInstallError('');
    try {
      const selected = await chooseDirectory(`${t('game.installPath')}: ${game.name}`, effectiveSettings.installPath);
      if (!selected) return;
      setGameOverride(game.id, true);
      patchGame(game.id, { installPath: selected });
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setPathBusy(false);
    }
  };

  const importExistingInstallation = async () => {
    if (pathBusy) return;
    setPathBusy(true);
    setInstallError('');
    try {
      const selected = await chooseDirectory(`${game.name}: ${t('game.existingInstall')}`, effectiveSettings.installPath);
      if (!selected) return;
      setGameOverride(game.id, true);
      patchGame(game.id, { installPath: selected });

      if (game.id === 'genshin') {
        const current = resolveGameSettings(game.id, globalSettings, {
          ...(gameSettings ?? { gameId: game.id, overrideEnabled: true }),
          installPath: selected,
          overrideEnabled: true,
        });
        const detected = await postInstallDefaults(game.id, selected, current.prefixPath);
        if (!detected.executablePath) {
          throw new Error(t('game.noExe'));
        }
        applyDetectedGameDefaults(game.id, detected.settings);
        patchGame(game.id, {
          installPath: selected,
          executablePath: detected.executablePath,
          prefixPath: detected.settings.prefixPath || current.prefixPath,
          installationDetectedAt: new Date().toISOString(),
        });
      }
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setPathBusy(false);
    }
  };

  const install = async () => {
    setBusy(true);
    setInstallError('');
    try {
      const result = await beginInstall(game.id, effectiveSettings.installPath);
      upsertJob(result);
      // The real downloader will return/emit `ready` when installation is complete.
      // At that point Genshin's post-install profile is applied automatically.
      if (game.id === 'genshin' && result.phase === 'ready') {
        try {
          const current = resolveGameSettings(game.id, globalSettings, gameSettings);
          const defaults = await postInstallDefaults(game.id, current.installPath, current.prefixPath);
          setGameOverride(game.id, true);
          applyDetectedGameDefaults(game.id, defaults.settings);
        } catch (postInstallError) {
          console.warn('Genshin post-install defaults could not be applied:', postInstallError);
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setInstallError(message);
    } finally {
      setBusy(false);
    }
  };

  const xxmiImporterReady = game.xxmiImporter
    ? (xxmiStatus?.importers.find((entry) => entry.id === game.xxmiImporter)?.installed ?? false)
    : false;

  const play = async () => {
    if (!effectiveSettings.executablePath || launchBusy) return;
    setLaunchBusy(true);
    setInstallError('');
    try {
      if (effectiveSettings.xxmiEnabled && game.xxmiImporter) {
        if (!xxmiStatus?.installed) throw new Error('XXMI ist aktiviert, aber noch nicht installiert.');
        if (!xxmiImporterReady) throw new Error(`${game.xxmiImporter} ist noch nicht eingerichtet.`);
        await launchXxmi(globalSettings, game.xxmiImporter, true);
        return;
      }

      await launchGame({
        gameId: game.id,
        executablePath: effectiveSettings.executablePath,
        runtime: effectiveSettings.runtime,
        runnerVersion: effectiveSettings.runnerVersion,
        runnerPath: effectiveSettings.runnerPath,
        prefixPath: effectiveSettings.prefixPath,
        launchArguments: effectiveSettings.launchArguments,
        environmentVariables: effectiveSettings.environmentVariables,
        gamescopeEnabled: effectiveSettings.gamescopeEnabled,
        gameModeEnabled: effectiveSettings.gameModeEnabled,
        mangoHudEnabled: effectiveSettings.mangoHudEnabled,
        preventSleep: effectiveSettings.preventSleep,
      });
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setLaunchBusy(false);
    }
  };

  const handleXxmiAction = async () => {
    if (!game.xxmiImporter || xxmiActionBusy) return;
    setXxmiActionBusy(true);
    setInstallError('');
    try {
      if (!xxmiStatus?.installed) {
        const installed = await installXxmi(globalSettings);
        if (!installed?.installed) throw new Error('XXMI konnte nicht installiert werden.');
        return;
      }
      if (!xxmiImporterReady) {
        await launchXxmi(globalSettings, game.xxmiImporter, false);
        await refreshXxmi(globalSettings);
        return;
      }
      patchGame(game.id, { xxmiEnabled: !effectiveSettings.xxmiEnabled });
    } catch (error) {
      setInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setXxmiActionBusy(false);
    }
  };

  const xxmiButtonLabel = !xxmiStatus?.installed
    ? 'XXMI INSTALLIEREN'
    : !xxmiImporterReady
      ? `${game.xxmiImporter ?? 'XXMI'} SETUP`
      : `XXMI ${effectiveSettings.xxmiEnabled ? 'ON' : 'OFF'}`;

  const installationRecognized = Boolean(effectiveSettings.executablePath);
  const progress = Math.round((job?.progress ?? 0) * 100);
  const showProgress = Boolean(job && !['ready', 'error', 'cancelled'].includes(job.phase));

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
          <span className="stage-version">v0.13.2</span>
        </div>
        <div className="topbar-actions">
          <LanguageSwitcher />
          <span className="network-pill"><i /> Online</span>
          <button className="icon-button" onClick={() => openGameSettings(game.id)} title={t('game.settings')}>
            <Settings2 size={18} />
          </button>
          <button className="icon-button" title={t('game.more')}><MoreHorizontal size={19} /></button>
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

        {installError && (
          <div className="hero-notice download-error"><ShieldAlert size={16} /> {installError}</div>
        )}

        <div className="hero-install-path">
          <div>
            <span>{t('game.installPath')}</span>
            <strong title={effectiveSettings.installPath}>{effectiveSettings.installPath}</strong>
          </div>
          <button className="ghost-button compact" onClick={() => void chooseInstallFolder()} disabled={pathBusy}>
            <FolderOpen size={16} /> {pathBusy ? t('game.opening') : t('game.chooseFolder')}
          </button>
        </div>

        {showProgress && job ? (
          <div className="inline-download">
            <div className="inline-download-head">
              <div>
                <span>{job.phase}</span>
                <strong>{progress}%</strong>
              </div>
              <small>{fmt(job.downloadedBytes)} / {fmt(job.totalBytes)} · {fmt(job.speedBytes)}/s</small>
            </div>
            <div className="hero-progress"><i style={{ width: `${progress}%` }} /></div>
            {job.message && <div className="download-message">{job.message}</div>}
          </div>
        ) : (
          <div className="hero-actions">
            <button
              className="play-button"
              onClick={() => installationRecognized ? void play() : void install()}
              disabled={busy || launchBusy}
            >
              {installationRecognized || game.status === 'ready' ? <Play size={20} fill="currentColor" /> : <Download size={20} />}
              {launchBusy ? t('common.starting') : busy ? t('common.preparing') : installationRecognized ? t('common.play') : game.status === 'ready' ? t('common.play') : t('common.install')}
            </button>
            <button className="ghost-button" onClick={() => void importExistingInstallation()} disabled={pathBusy}>
              <FolderOpen size={18} /> {t('game.existingInstall')}
            </button>
            {game.modPolicy !== 'disabled' && <button className="ghost-button" onClick={() => setSection('mods')}><Puzzle size={18} /> Mods</button>}
            {game.xxmiImporter && (
              <button
                className={`ghost-button xxmi-quick-toggle ${effectiveSettings.xxmiEnabled && xxmiImporterReady ? 'active' : ''}`}
                onClick={() => void handleXxmiAction()}
                disabled={xxmiActionBusy}
                title={!xxmiStatus?.installed ? 'XXMI über GachaHub installieren' : !xxmiImporterReady ? `${game.xxmiImporter} einmalig in XXMI einrichten` : `${game.xxmiImporter} ${effectiveSettings.xxmiEnabled ? 'deaktivieren' : 'aktivieren'}`}
              >
                <Sparkles size={18} /> {xxmiActionBusy ? 'XXMI…' : xxmiButtonLabel}
              </button>
            )}
          </div>
        )}
      </section>

      <section className="stage-footer">
        <div className="stage-stat"><span>{t('game.status')}</span><strong>{job ? job.phase : installationRecognized ? t('game.installDetected') : game.status === 'ready' ? t('common.ready') : t('common.notInstalled')}</strong></div>
        <div className="stage-stat"><span>{t('game.installType')}</span><strong>{game.installModes[0]}</strong></div>
        <div className="stage-stat"><span>{t('game.runtime')}</span><strong>{effectiveSettings.runtime} · {effectiveSettings.runnerVersion || 'default'}</strong></div>
        <div className="stage-stat"><span>{t('game.playtime')}</span><strong>—</strong></div>
      </section>
    </main>
  );
}
