import {
  AlertTriangle,
  CheckCircle2,
  Archive,
  FolderInput,
  FolderSync,
  Import,
  LoaderCircle,
  Puzzle,
  RefreshCw,
  Save,
  Search,
  Settings2,
  ShieldAlert,
  SlidersHorizontal,
  Sparkles,
  Unplug,
} from 'lucide-react';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { useEffect, useMemo, useState } from 'react';
import type { CSSProperties } from 'react';
import { useI18n } from '../i18n';
import type { ModEntry, ModSnapshot } from '../lib/mods';
import { resolveGameSettings } from '../lib/settings';
import { getCompatibilityPreset } from '../data/compatibilityPresets';
import {
  adoptMod,
  disableMod,
  enableMod,
  archiveToolStatus,
  importModSource,
  readLegacyGmmConfig,
  scanMods,
  syncModProfile,
} from '../lib/tauri';
import { useLauncherStore } from '../stores/useLauncherStore';
import { useModStore } from '../stores/useModStore';
import { useSettingsStore } from '../stores/useSettingsStore';
import { useXxmiStore } from '../stores/useXxmiStore';

function fmt(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  return `${(bytes / Math.pow(1024, i)).toFixed(i >= 3 ? 1 : 0)} ${units[i]}`;
}

function statusLabel(mod: ModEntry, t: (key: string) => string) {
  if (mod.status === 'enabled') return t('common.active');
  if (mod.status === 'conflict') return t('common.conflicts');
  if (mod.status === 'broken') return t('common.broken');
  return t('common.inactive');
}

export function Mods() {
  const { t } = useI18n();
  const games = useLauncherStore((state) => state.games);
  const defaultGameId = useLauncherStore((state) => state.selectedGameId);
  const openGameSettings = useLauncherStore((state) => state.openGameSettings);
  const global = useSettingsStore((state) => state.global);
  const gameSettings = useSettingsStore((state) => state.games);
  const patchGame = useSettingsStore((state) => state.patchGame);
  const setGameOverride = useSettingsStore((state) => state.setGameOverride);
  const profiles = useModStore((state) => state.profiles);
  const activeProfiles = useModStore((state) => state.activeProfiles);
  const saveProfile = useModStore((state) => state.saveProfile);
  const setActiveProfile = useModStore((state) => state.setActiveProfile);
  const xxmiStatus = useXxmiStore((state) => state.status);
  const xxmiBusy = useXxmiStore((state) => state.busy);
  const installXxmi = useXxmiStore((state) => state.install);
  const launchXxmi = useXxmiStore((state) => state.launch);
  const refreshXxmi = useXxmiStore((state) => state.refresh);

  const [selectedId, setSelectedId] = useState(defaultGameId);
  const [snapshot, setSnapshot] = useState<ModSnapshot>();
  const [query, setQuery] = useState('');
  const [filter, setFilter] = useState<'all' | 'enabled' | 'disabled' | 'problem'>('all');
  const [busyId, setBusyId] = useState<string>();
  const [error, setError] = useState<string>();
  const [sourcePath, setSourcePath] = useState('');
  const [legacyPath, setLegacyPath] = useState('');
  const [profileName, setProfileName] = useState('Default');
  const [dragging, setDragging] = useState(false);
  const [archiveTool, setArchiveTool] = useState('checking');
  const [xxmiMessage, setXxmiMessage] = useState('');

  const selected = useMemo(() => games.find((game) => game.id === selectedId) ?? games[0], [games, selectedId]);
  const stored = selected ? gameSettings[selected.id] : undefined;
  const effective = selected ? resolveGameSettings(selected.id, global, stored) : undefined;
  const currentProfile = selected ? activeProfiles[selected.id] ?? 'Default' : 'Default';
  const gameProfiles = selected ? profiles[selected.id] ?? {} : {};
  const compatibility = selected ? getCompatibilityPreset(selected.id) : undefined;
  const modBackend = compatibility?.modding.backend ?? (selected?.xxmiImporter ? 'xxmi' : 'library-only');
  const deploymentAvailable = Boolean(selected?.xxmiImporter && selected?.modPolicy === 'supported' && modBackend === 'xxmi');
  const xxmiImporterReady = selected?.xxmiImporter
    ? (xxmiStatus?.importers.find((entry) => entry.id === selected.xxmiImporter)?.installed ?? false)
    : false;

  const handleXxmiAction = async (quickLaunch: boolean) => {
    if (!selected?.xxmiImporter) return;
    setXxmiMessage('');
    try {
      if (!xxmiStatus?.installed) {
        const installed = await installXxmi(global);
        setXxmiMessage(installed?.installed ? 'XXMI installiert. Importer-Setup kann jetzt geöffnet werden.' : 'XXMI Installation fehlgeschlagen.');
        return;
      }
      if (!xxmiImporterReady) {
        await launchXxmi(global, selected.xxmiImporter, false);
        setXxmiMessage(`${selected.xxmiImporter} Setup geöffnet.`);
        await refreshXxmi(global);
        return;
      }
      if (quickLaunch) {
        await launchXxmi(global, selected.xxmiImporter, global.xxmiNoGui);
        setXxmiMessage(`${selected.xxmiImporter} gestartet`);
      } else {
        patchGame(selected.id, { xxmiEnabled: !(effective?.xxmiEnabled ?? false) });
      }
    } catch (error) {
      setXxmiMessage(String(error));
    }
  };

  const refresh = async () => {
    if (!selected || !effective) return;
    setError(undefined);
    try {
      setSnapshot(await scanMods(selected.id, effective.modLibraryPath, effective.activeModsPath));
    } catch (e) {
      setError(String(e));
      setSnapshot(undefined);
    }
  };

  useEffect(() => {
    setSnapshot(undefined);
    setError(undefined);
    if (selected) void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected?.id, effective?.modLibraryPath, effective?.activeModsPath]);

  useEffect(() => {
    void archiveToolStatus().then(setArchiveTool).catch(() => setArchiveTool('not-found'));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        setDragging(true);
      } else if (event.payload.type === 'drop') {
        setDragging(false);
        if (selected) {
          const paths = event.payload.paths;
          void (async () => {
            for (const path of paths) await importSource(path);
          })();
        }
      } else {
        setDragging(false);
      }
    }).then((fn) => { unlisten = fn; });
    return () => unlisten?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected?.id, effective?.modLibraryPath, effective?.activeModsPath]);

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return (snapshot?.entries ?? []).filter((entry) => {
      const matchesSearch = !needle || entry.name.toLowerCase().includes(needle) || entry.relativePath.toLowerCase().includes(needle);
      const matchesFilter = filter === 'all'
        || (filter === 'enabled' && entry.status === 'enabled')
        || (filter === 'disabled' && entry.status === 'disabled')
        || (filter === 'problem' && (entry.status === 'conflict' || entry.status === 'broken'));
      return matchesSearch && matchesFilter;
    });
  }, [snapshot, query, filter]);

  const toggle = async (entry: ModEntry) => {
    if (!selected || !effective || !deploymentAvailable) return;
    setBusyId(entry.id);
    setError(undefined);
    try {
      const next = entry.enabled
        ? await disableMod(selected.id, entry.id, effective.modLibraryPath, effective.activeModsPath)
        : await enableMod(selected.id, entry.id, effective.modLibraryPath, effective.activeModsPath, effective.modDeploymentMode);
      setSnapshot(next);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(undefined);
    }
  };

  const adopt = async (entry: ModEntry) => {
    if (!selected || !effective || !deploymentAvailable) return;
    setBusyId(entry.id);
    setError(undefined);
    try {
      setSnapshot(await adoptMod(selected.id, entry.id, effective.modLibraryPath, effective.activeModsPath));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(undefined);
    }
  };

  const importSource = async (path = sourcePath) => {
    if (!selected || !effective || !path.trim()) return;
    setBusyId('__import__');
    setError(undefined);
    try {
      setSnapshot(await importModSource(selected.id, path.trim(), effective.modLibraryPath, effective.activeModsPath));
      setSourcePath('');
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(undefined);
    }
  };

  const importLegacyConfig = async () => {
    if (!selected || !legacyPath.trim()) return;
    setBusyId('__legacy__');
    setError(undefined);
    try {
      const config = await readLegacyGmmConfig(legacyPath.trim());
      setGameOverride(selected.id, true);
      patchGame(selected.id, {
        legacyGmmConfigPath: config.configPath,
        ...(config.libraryPath ? { modLibraryPath: config.libraryPath } : {}),
        ...(config.activeModsPath ? { activeModsPath: config.activeModsPath } : {}),
      });
      setLegacyPath(config.configPath);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(undefined);
    }
  };

  const saveCurrentProfile = () => {
    if (!selected || !snapshot) return;
    const name = profileName.trim() || 'Default';
    saveProfile(selected.id, name, snapshot.entries.filter((m) => m.enabled && m.managed).map((m) => m.id));
    setActiveProfile(selected.id, name);
  };

  const applyProfile = async (name: string) => {
    if (!selected || !effective) return;
    if (!deploymentAvailable) {
      setActiveProfile(selected.id, name);
      setProfileName(name);
      return;
    }
    const enabledIds = gameProfiles[name] ?? [];
    setBusyId('__profile__');
    setError(undefined);
    try {
      setSnapshot(await syncModProfile(selected.id, enabledIds, effective.modLibraryPath, effective.activeModsPath, effective.modDeploymentMode));
      setActiveProfile(selected.id, name);
      setProfileName(name);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyId(undefined);
    }
  };

  return (
    <main className="page-shell">
      <header className="page-header">
        <div><span className="page-eyebrow">{t('mods.eyebrow')}</span><h1>{t('mods.title')}</h1><p>{t('mods.subtitle')}</p></div>
        <button className="header-action" onClick={() => void refresh()} disabled={busyId === '__scan__'}><RefreshCw size={17}/> {t('common.search')}</button>
      </header>

      <div className="mod-layout mod-layout-v5">
        <aside className="mod-game-list">
          {games.map((game) => (
            <button key={game.id} onClick={() => setSelectedId(game.id)} className={game.id === selected?.id ? 'active' : ''}>
              <span className="mod-game-icon" style={{ '--mod-accent': game.accent } as CSSProperties}>{game.shortName}</span>
              <div><strong>{game.name}</strong><small>{getCompatibilityPreset(game.id)?.modding.label ?? game.modPolicy}</small></div>
            </button>
          ))}
        </aside>

        <section className="mod-workspace">
          <div className="mod-hero" style={{ '--mod-accent': selected?.accent } as CSSProperties}>
            <Puzzle size={23}/>
            <div><span>{t('mods.activeProfile')}</span><strong>{selected?.name} · {currentProfile}</strong></div>
            {selected?.xxmiImporter && (
              <button
                className={effective?.xxmiEnabled && xxmiImporterReady ? 'xxmi-hero-button active' : 'xxmi-hero-button'}
                onClick={() => void handleXxmiAction(false)}
                disabled={xxmiBusy}
                title={!xxmiStatus?.installed ? 'XXMI installieren' : !xxmiImporterReady ? `${selected.xxmiImporter} Setup öffnen` : `${selected.xxmiImporter} umschalten`}
              ><Sparkles size={15}/> {!xxmiStatus?.installed ? 'XXMI installieren' : !xxmiImporterReady ? `${selected.xxmiImporter} Setup` : `${selected.xxmiImporter} ${effective?.xxmiEnabled ? 'ON' : 'OFF'}`}</button>
            )}
            <button onClick={() => selected && openGameSettings(selected.id)}><Settings2 size={16}/> Settings</button>
          </div>

          {selected?.modPolicy !== 'supported' && (
            <div className="mod-warning"><ShieldAlert size={20}/><div><strong>{t('mods.restricted')}</strong><span>Für dieses Spiel ist die Mod-Policy aktuell {selected?.modPolicy}. Library, Import, Suche und Profile funktionieren; GachaHub aktiviert aber keinen Loader automatisch.</span></div></div>
          )}

          {modBackend === 'library-only' && (
            <div className="mod-connection-card">
              <div className="mod-connection-head">
                <div><Archive size={18}/><span><strong>{t('mods.libraryOnly')}</strong><small>Mods können für {selected?.name} organisiert, importiert und profiliert werden. Ein sicherer Loader-/Deployment-Adapter ist noch nicht hinterlegt.</small></span></div>
                <span className="backend-pill">Library only</span>
              </div>
            </div>
          )}

          <section className="mod-connection-card">
            <div className="mod-connection-head">
              <div><FolderSync size={18}/><span><strong>{t('mods.gameLibrary')}</strong><small>Jedes Spiel besitzt eine eigene Library. XXMI-Titel können zusätzlich einen Active-Mods-Ordner verwenden.</small></span></div>
              <span className={`backend-pill ${deploymentAvailable && effective?.activeModsPath ? 'online' : ''}`}>{deploymentAvailable ? (effective?.activeModsPath ? 'Verbunden' : 'Setup nötig') : 'Library'}</span>
            </div>
            <div className="mod-path-grid">
              <label><span>Library</span><input value={effective?.modLibraryPath ?? ''} readOnly /></label>
              <label><span>Active Mods / Loader Mods</span><input value={deploymentAvailable ? effective?.activeModsPath ?? '' : ''} readOnly placeholder={deploymentAvailable ? 'In Game Settings setzen' : 'Für diesen Adapter nicht erforderlich'} /></label>
            </div>
            {selected?.id === 'genshin' && (
              <div className="legacy-import-row">
                <input value={legacyPath} onChange={(e) => setLegacyPath(e.target.value)} placeholder="Pfad zu bestehender GMM config.json oder GMM-Ordner" />
                <button onClick={() => void importLegacyConfig()} disabled={busyId === '__legacy__'}>{busyId === '__legacy__' ? <LoaderCircle className="spin" size={15}/> : <Import size={15}/>} GMM Config übernehmen</button>
              </div>
            )}
          </section>

          {selected?.xxmiImporter && effective && (
            <section className="xxmi-runtime-card">
              <div><Sparkles size={18}/><span><strong>{t('mods.xxmiIntegration')}</strong><small>{selected.xxmiImporter} · {!xxmiStatus?.installed ? 'XXMI fehlt' : !xxmiImporterReady ? 'Importer Setup nötig' : effective.xxmiEnabled ? 'für dieses Spiel aktiviert' : 'bereit'}</small></span></div>
              <button
                disabled={xxmiBusy}
                onClick={() => void handleXxmiAction(true)}
              >{xxmiBusy ? 'Bitte warten…' : !xxmiStatus?.installed ? 'XXMI installieren' : !xxmiImporterReady ? `${selected.xxmiImporter} Setup öffnen` : 'XXMI starten'}</button>
              {xxmiMessage && <small className="xxmi-runtime-message">{xxmiMessage}</small>}
            </section>
          )}

          {error && <div className="mod-error"><AlertTriangle size={17}/><span>{error}</span></div>}

          {deploymentAvailable && !effective?.activeModsPath ? (
            <div className="mod-warning"><Unplug size={20}/><div><strong>{t('mods.activePathMissing')}</strong><span>Öffne die Game Settings und setze den XXMI/3DMigoto Mods-Ordner. Die Library kann bereits gescannt werden, aber Aktivieren/Deaktivieren bleibt gesperrt.</span></div></div>
          ) : null}

          <section className="mod-toolbar-v5">
            <div className="mod-search"><Search size={15}/><input value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t('mods.search')} /></div>
            <div className="mod-filter-group">
              {(['all','enabled','disabled','problem'] as const).map((value) => <button key={value} className={filter === value ? 'active' : ''} onClick={() => setFilter(value)}>{value === 'all' ? t('common.all') : value === 'enabled' ? t('common.active') : value === 'disabled' ? t('common.inactive') : t('common.problems')}</button>)}
            </div>
          </section>

          <section className="mod-stats-grid">
            <div><span>{t('common.total')}</span><strong>{snapshot?.stats.total ?? 0}</strong></div>
            <div><span>{t('common.active')}</span><strong>{snapshot?.stats.enabled ?? 0}</strong></div>
            <div><span>{t('common.conflicts')}</span><strong>{snapshot?.stats.conflicts ?? 0}</strong></div>
            <div><span>{t('common.broken')}</span><strong>{snapshot?.stats.broken ?? 0}</strong></div>
          </section>

          <section className="mod-profile-bar">
            <div><SlidersHorizontal size={16}/><select value={currentProfile} onChange={(e) => void applyProfile(e.target.value)}><option value="Default">Default</option>{Object.keys(gameProfiles).filter((x) => x !== 'Default').map((name) => <option key={name} value={name}>{name}</option>)}</select></div>
            <div><input value={profileName} onChange={(e) => setProfileName(e.target.value)} placeholder={t('mods.profileName')}/><button onClick={saveCurrentProfile} disabled={!snapshot}><Save size={15}/> {t('mods.saveState')}</button></div>
          </section>

          <section className={`mod-import-dropzone ${dragging ? 'dragging' : ''}`}>
            <div className="mod-drop-copy">
              <Archive size={21}/>
              <span><strong>{t('mods.dropHere')}</strong><small>Ordner, ZIP, 7z oder RAR · ZIP ist integriert; 7z/RAR nutzt 7-Zip ({archiveTool === 'not-found' ? 'nicht gefunden' : archiveTool === 'checking' ? 'wird geprüft' : archiveTool}).</small></span>
            </div>
            <div className="mod-import-controls">
              <input value={sourcePath} onChange={(e) => setSourcePath(e.target.value)} placeholder="/pfad/zu/mod.zip oder Mod-Ordner" />
              <button onClick={() => void importSource()} disabled={!sourcePath.trim() || busyId === '__import__'}>{busyId === '__import__' ? <LoaderCircle className="spin" size={15}/> : <FolderInput size={15}/>} {t('mods.import')}</button>
            </div>
          </section>

          <div className="mod-list-v5">
            {visible.length === 0 ? (
              <div className="mod-empty-v5"><Puzzle size={28}/><strong>{t('mods.none')}</strong><span>{t('mods.noneHint')}</span></div>
            ) : visible.map((entry) => (
              <article key={entry.id} className={`mod-row-v5 status-${entry.status}`}>
                <div className="mod-row-state">{entry.status === 'enabled' ? <CheckCircle2 size={18}/> : entry.status === 'conflict' || entry.status === 'broken' ? <AlertTriangle size={18}/> : <Puzzle size={18}/>}</div>
                <div className="mod-row-main"><strong>{entry.name}</strong><span>{entry.relativePath}</span>{entry.conflictReason && <small>{entry.conflictReason}</small>}</div>
                <div className="mod-row-meta"><span>{entry.iniCount} INI</span><span>{fmt(entry.sizeBytes)}</span><b>{statusLabel(entry, t)}</b></div>
                {entry.status === 'conflict' && entry.enabled && !entry.managed ? (
                  <button className="mod-toggle-btn adopt" disabled={!deploymentAvailable || busyId === entry.id || !effective?.activeModsPath} onClick={() => void adopt(entry)}>
                    {busyId === entry.id ? <LoaderCircle className="spin" size={15}/> : 'Übernehmen'}
                  </button>
                ) : (
                  <button className={`mod-toggle-btn ${entry.enabled ? 'enabled' : ''}`} disabled={!deploymentAvailable || busyId === entry.id || entry.status === 'broken' || entry.status === 'conflict' || !effective?.activeModsPath} onClick={() => void toggle(entry)}>
                    {busyId === entry.id ? <LoaderCircle className="spin" size={15}/> : deploymentAvailable ? (entry.enabled ? t('common.inactive') : t('common.active')) : 'Library'}
                  </button>
                )}
              </article>
            ))}
          </div>
        </section>
      </div>
    </main>
  );
}
