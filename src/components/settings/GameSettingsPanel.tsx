import {
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  Gauge,
  Info,
  Play,
  RotateCcw,
  ShieldCheck,
  Sparkles,
} from 'lucide-react';
import { useState, type CSSProperties } from 'react';
import type { Game } from '../../lib/types';
import {
  resolveGameSettings,
  type GameSettings,
  type ModDeploymentMode,
  type RuntimeKind,
} from '../../lib/settings';
import { useI18n } from '../../i18n';
import { getCompatibilityPreset } from '../../data/compatibilityPresets';
import { chooseDirectory, postInstallDefaults, type PostInstallResult } from '../../lib/tauri';
import { useSettingsStore } from '../../stores/useSettingsStore';
import { useXxmiStore } from '../../stores/useXxmiStore';
import { RunnerInput } from './RunnerInput';

interface GameSettingsPanelProps {
  game: Game;
}

function mergeLines(current: string | undefined, additional: string | undefined): string {
  const lines = [...(current ?? '').split('\n'), ...(additional ?? '').split('\n')]
    .map((line) => line.trim())
    .filter(Boolean);
  return [...new Set(lines)].join('\n');
}

function mergeArguments(current: string | undefined, additional: string | undefined): string {
  const value = `${current ?? ''} ${additional ?? ''}`.trim();
  if (!value) return '';
  return [...new Set(value.split(/\s+/).filter(Boolean))].join(' ');
}

function stateClass(state?: string): string {
  if (state === 'native' || state === 'playable') return 'good';
  if (state === 'playable-with-caveats') return 'warn';
  if (state === 'unsupported') return 'bad';
  return 'neutral';
}

export function GameSettingsPanel({ game }: GameSettingsPanelProps) {
  const { t } = useI18n();
  const global = useSettingsStore((state) => state.global);
  const storedGame = useSettingsStore((state) => state.games[game.id]);
  const setGameOverride = useSettingsStore((state) => state.setGameOverride);
  const patchGame = useSettingsStore((state) => state.patchGame);
  const applyDetectedGameDefaults = useSettingsStore((state) => state.applyDetectedGameDefaults);
  const resetGame = useSettingsStore((state) => state.resetGame);
  const overrideEnabled = storedGame?.overrideEnabled ?? true;
  const effective = resolveGameSettings(game.id, global, storedGame);
  const preset = getCompatibilityPreset(game.id);
  const [xxmiMessage, setXxmiMessage] = useState('');
  const [xxmiBusy, setXxmiBusy] = useState(false);
  const [postInstallBusy, setPostInstallBusy] = useState(false);
  const [postInstallResult, setPostInstallResult] = useState<PostInstallResult | null>(null);
  const [postInstallMessage, setPostInstallMessage] = useState('');
  const [pathPickerBusy, setPathPickerBusy] = useState(false);
  const xxmiStatus = useXxmiStore((state) => state.status);
  const xxmiManagerBusy = useXxmiStore((state) => state.busy);
  const installManagedXxmi = useXxmiStore((state) => state.install);
  const launchManagedXxmi = useXxmiStore((state) => state.launch);
  const refreshManagedXxmi = useXxmiStore((state) => state.refresh);

  const patch = (value: Partial<GameSettings>) => patchGame(game.id, value);
  const modBackend = preset?.modding.backend ?? (game.xxmiImporter ? 'xxmi' : 'library-only');
  const deploymentAvailable = modBackend === 'xxmi';

  const applyRecommendedPreset = () => {
    if (!preset) return;
    setGameOverride(game.id, true);
    const useXxmiArgs = (storedGame?.xxmiEnabled ?? false) ? preset.linux.xxmiLaunchArguments : undefined;
    patchGame(game.id, {
      overrideEnabled: true,
      runtime: preset.linux.recommendedRuntime,
      // The preset describes a preferred family, but it must not pretend that a
      // concrete local build is installed. With no path, UMU can choose its managed
      // runtime until the user selects an installed runner below.
      runnerVersion: '',
      runnerPath: '',
      launchArguments: mergeArguments('', useXxmiArgs),
      environmentVariables: preset.linux.environmentVariables,
      gamescopeEnabled: false,
      gameModeEnabled: false,
      mangoHudEnabled: false,
      preventSleep: true,
      compatibilityPresetAppliedAt: preset.checkedAt,
    });
  };

  const applyOptionalWorkaround = () => {
    if (!preset?.linux.optionalEnvironmentVariables) return;
    setGameOverride(game.id, true);
    patchGame(game.id, {
      overrideEnabled: true,
      environmentVariables: mergeLines(
        storedGame?.environmentVariables,
        preset.linux.optionalEnvironmentVariables,
      ),
    });
  };

  const applyXxmiRequirements = () => {
    if (!preset?.linux.xxmiLaunchArguments) return;
    setGameOverride(game.id, true);
    patchGame(game.id, {
      overrideEnabled: true,
      launchArguments: mergeArguments(
        storedGame?.launchArguments,
        preset.linux.xxmiLaunchArguments,
      ),
    });
  };

  const applyGenshinPostInstall = async () => {
    if (game.id !== 'genshin') return;
    setPostInstallBusy(true);
    setPostInstallMessage('');
    try {
      const result = await postInstallDefaults(game.id, effective.installPath, effective.prefixPath);
      setGameOverride(game.id, true);
      applyDetectedGameDefaults(game.id, result.settings);
      setPostInstallResult(result);
      setPostInstallMessage('Genshin Standardprofil wurde erkannt und übernommen.');
    } catch (error) {
      setPostInstallMessage(String(error));
    } finally {
      setPostInstallBusy(false);
    }
  };

  const browseInstallPath = async () => {
    if (!overrideEnabled || pathPickerBusy) return;
    setPathPickerBusy(true);
    try {
      const selected = await chooseDirectory(`Installationsordner für ${game.name}`, effective.installPath);
      if (selected) patch({ installPath: selected });
    } finally {
      setPathPickerBusy(false);
    }
  };


  return (
    <div className="settings-panel-stack">
      <div className="settings-panel-header game-settings-header" style={{ '--game-accent': game.accent } as CSSProperties}>
        <div>
          <span className="eyebrow">GAME SETTINGS</span>
          <h2>{game.name}</h2>
          <p>{game.publisher} · Provider: {game.provider}</p>
        </div>
        <button className="secondary-btn" onClick={() => resetGame(game.id)}>
          <RotateCcw size={15} /> Overrides zurücksetzen
        </button>
      </div>

      {preset && (
        <section className="settings-section compatibility-section">
          <div className="settings-section-title compatibility-title-row">
            <div>
              <h3>{t('settings.compatibility')}</h3>
              <p>Recherche-Stand {preset.checkedAt}. Community-/Launcher-Kompatibilität, keine offizielle Linux-Zusage.</p>
            </div>
            <button className="recommended-preset-btn" onClick={applyRecommendedPreset}>
              <Sparkles size={15} /> {t('settings.applyRecommended')}
            </button>
          </div>

          <div className="compatibility-grid">
            <div className={`compatibility-card ${stateClass(preset.windows.state)}`}>
              <span>WINDOWS</span>
              <strong>{preset.windows.label}</strong>
              <small>{preset.windows.state}</small>
            </div>
            <div className={`compatibility-card ${stateClass(preset.linux.state)}`}>
              <span>LINUX</span>
              <strong>{preset.linux.label}</strong>
              <small>{preset.linux.state}</small>
            </div>
            <div className="compatibility-card neutral">
              <span>EMPFOHLENER RUNNER</span>
              <strong>{preset.linux.recommendedRunner}</strong>
              <small>{preset.linux.recommendedRuntime}</small>
            </div>
            <div className={`compatibility-card ${modBackend === 'xxmi' ? 'good' : 'neutral'}`}>
              <span>MOD BACKEND</span>
              <strong>{preset.modding.label}</strong>
              <small>{modBackend}</small>
            </div>
          </div>

          <div className="compatibility-details-grid">
            <div className="compatibility-detail-box">
              <strong><CheckCircle2 size={14} /> Runner-Kandidaten</strong>
              <div className="compatibility-chip-row">
                {preset.linux.runnerCandidates.map((runner) => <span key={runner}>{runner}</span>)}
              </div>
            </div>
            <div className="compatibility-detail-box">
              <strong><AlertTriangle size={14} /> Nicht bevorzugen / bekannte Regressionen</strong>
              {preset.linux.blockedRunners.length ? (
                <div className="compatibility-chip-row blocked">
                  {preset.linux.blockedRunners.map((runner) => <span key={runner}>{runner}</span>)}
                </div>
              ) : <small>Aktuell kein fester Block-Eintrag. Bei Problemen Runner wechseln.</small>}
            </div>
          </div>

          <div className="compatibility-notes">
            {preset.linux.notes.map((note) => (
              <div key={note}><Info size={13} /><span>{note}</span></div>
            ))}
          </div>

          {preset.linux.optionalEnvironmentVariables && (
            <div className="compatibility-action-row">
              <code>{preset.linux.optionalEnvironmentVariables}</code>
              <button className="secondary-btn" onClick={applyOptionalWorkaround}>
                Optionalen Workaround übernehmen
              </button>
            </div>
          )}
        </section>
      )}

      {game.id === 'genshin' && (
        <section className="settings-section post-install-section">
          <div className="settings-section-title compatibility-title-row">
            <div>
              <h3>{t('settings.postInstall')}</h3>
              <p>Nach der Genshin-Installation erkennt GachaHub die lokale Linux/Windows-Umgebung und übernimmt konservative Standardwerte.</p>
            </div>
            <button className="recommended-preset-btn" disabled={postInstallBusy} onClick={applyGenshinPostInstall}>
              <Sparkles size={15} /> {postInstallBusy ? 'Erkenne…' : t('settings.detectApply')}
            </button>
          </div>

          <div className="compatibility-notes">
            <div><Info size={13} /><span>Runner wird nur aus vorhandenen Installationen erkannt; fehlt ein passender Runner, bleibt die Auswahl auf Auto.</span></div>
            <div><ShieldCheck size={13} /><span>XXMI, Mods und optionale Anti-Cheat-/Timeout-Workarounds bleiben standardmäßig ausgeschaltet.</span></div>
          </div>

          {storedGame?.postInstallProfile && (
            <div className="inheritance-notice">
              <CheckCircle2 size={16} />
              <span>Aktiv: {storedGame.postInstallProfile} · zuletzt angewendet: {storedGame.postInstallAppliedAt ?? 'unbekannt'}</span>
            </div>
          )}

          {postInstallResult && (
            <div className="compatibility-details-grid">
              <div className="compatibility-detail-box">
                <strong>Erkannte Tools</strong>
                <div className="compatibility-chip-row">
                  <span>GameMode {postInstallResult.detected.gameMode ? '✓' : '—'}</span>
                  <span>Gamescope {postInstallResult.detected.gamescope ? '✓' : '—'}</span>
                  <span>MangoHud {postInstallResult.detected.mangoHud ? '✓' : '—'}</span>
                  <span>umu {postInstallResult.detected.umu ? '✓' : '—'}</span>
                </div>
              </div>
              <div className="compatibility-detail-box">
                <strong>Erkannter Startpunkt</strong>
                <small>{postInstallResult.executablePath || 'GenshinImpact.exe noch nicht gefunden'}</small>
              </div>
            </div>
          )}

          {postInstallResult?.warnings.map((warning) => (
            <div className="inheritance-notice" key={warning}><AlertTriangle size={15}/><span>{warning}</span></div>
          ))}
          {postInstallMessage && <div className="xxmi-inline-message">{postInstallMessage}</div>}
        </section>
      )}

      <section className="settings-section override-card">
        <label className="override-toggle">
          <span>
            <strong>{t('settings.gameSpecific')}: {game.name}</strong>
            <small>
              {overrideEnabled
                ? 'Game-spezifische Werte überschreiben die globalen Einstellungen.'
                : 'Das Spiel erbt aktuell alle kompatiblen globalen Einstellungen.'}
            </small>
          </span>
          <input
            type="checkbox"
            checked={overrideEnabled}
            onChange={(event) => setGameOverride(game.id, event.target.checked)}
          />
        </label>
      </section>

      {!overrideEnabled && (
        <div className="inheritance-notice">
          <Info size={17} />
          <span>
            Die Felder zeigen die aktuell geerbten Werte. Aktiviere den Override oben oder wende das Kompatibilitäts-Preset an.
          </span>
        </div>
      )}

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('settings.installation')}</h3>
          <p>Installationspfad und dateibezogene Optionen.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>{t('game.installPath')}</span>
            <div className="settings-path-row">
              <input
                disabled={!overrideEnabled}
                value={overrideEnabled ? storedGame?.installPath ?? '' : effective.installPath}
                placeholder={effective.installPath}
                onChange={(event) => patch({ installPath: event.target.value })}
              />
              <button type="button" className="secondary-btn path-browse-btn" disabled={!overrideEnabled || pathPickerBusy} onClick={() => void browseInstallPath()}>
                <FolderOpen size={15} /> {pathPickerBusy ? 'Öffne…' : 'Durchsuchen'}
              </button>
            </div>
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('settings.verifyBeforeLaunch')}</strong><small>{t('global.verifyHint')}</small></span>
            <input
              type="checkbox"
              disabled={!overrideEnabled}
              checked={overrideEnabled ? storedGame?.verifyBeforeLaunch ?? global.verifyBeforeLaunch : effective.verifyBeforeLaunch}
              onChange={(event) => patch({ verifyBeforeLaunch: event.target.checked })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('settings.runtime')}</h3>
          <p>Game-spezifische Linux-Kompatibilitätsumgebung.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>{t('settings.runtime')}</span>
            <select
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.runtime ?? global.defaultRuntime : effective.runtime}
              onChange={(event) => patch({ runtime: event.target.value as RuntimeKind })}
            >
              <option value="auto">Auto</option>
              <option value="native">Native</option>
              <option value="umu">umu</option>
              <option value="ge-proton">GE-Proton</option>
              <option value="proton">Proton</option>
              <option value="wine">Wine</option>
            </select>
          </label>
          <div className="settings-field">
            <span>{t('settings.runner')}</span>
            <RunnerInput
              inputId={`game-runner-${game.id}`}
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.runnerVersion ?? '' : effective.runnerVersion}
              pathValue={overrideEnabled ? storedGame?.runnerPath ?? '' : effective.runnerPath}
              placeholder={preset?.linux.recommendedRunner ?? effective.runnerVersion}
              onChange={(value, path, runtime) => patch({ runnerVersion: value, runnerPath: path, runtime })}
            />
          </div>
          <label className="settings-field settings-field-wide">
            <span>{t('settings.prefix')}</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.prefixPath ?? '' : effective.prefixPath}
              placeholder={effective.prefixPath}
              onChange={(event) => patch({ prefixPath: event.target.value })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('settings.linuxExtras')}</h3>
          <p>Pro Spiel steuerbare Hilfen für Windowing, Performance und Session-Verhalten.</p>
        </div>
        <div className="settings-form-grid">
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('settings.gamescope')}</strong><small>Kann besonders bei Fullscreen-/Input-Problemen helfen.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.gamescopeEnabled} onChange={(event) => patch({ gamescopeEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('settings.gamemode')}</strong><small>gamemoderun beim späteren Launch-Backend verwenden.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.gameModeEnabled} onChange={(event) => patch({ gameModeEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('settings.mangohud')}</strong><small>Performance-Overlay für diesen Titel.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.mangoHudEnabled} onChange={(event) => patch({ mangoHudEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('settings.preventSleep')}</strong><small>System während des Spielens wach halten.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.preventSleep} onChange={(event) => patch({ preventSleep: event.target.checked })} />
          </label>
          <label className="settings-field">
            <span>{t('settings.fpsLimit')}</span>
            <input
              type="number"
              min={0}
              disabled={!overrideEnabled}
              placeholder={t('common.unlimited')}
              value={overrideEnabled ? storedGame?.fpsLimit ?? '' : effective.fpsLimit ?? ''}
              onChange={(event) => patch({ fpsLimit: event.target.value ? Math.max(0, Number(event.target.value)) : null })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('settings.launch')}</h3>
          <p>Startargumente und Umgebungsvariablen.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>{t('settings.launchArgs')}</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.launchArguments ?? '' : ''}
              placeholder={preset?.linux.launchArguments || 'z. B. -fullscreen'}
              onChange={(event) => patch({ launchArguments: event.target.value })}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>{t('settings.envVars')}</span>
            <textarea
              disabled={!overrideEnabled}
              rows={5}
              value={overrideEnabled ? storedGame?.environmentVariables ?? '' : ''}
              placeholder={'Eine Variable pro Zeile, z. B.\nMANGOHUD=1\nDXVK_LOG_LEVEL=none'}
              onChange={(event) => patch({ environmentVariables: event.target.value })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section xxmi-settings-section">
        <div className="settings-section-title">
          <h3>XXMI</h3>
          <p>XXMI wird von GachaHub verwaltet. Pro Spiel bleibt nur der passende Model Importer aktivierbar.</p>
        </div>
        {game.xxmiImporter ? (
          <div className="settings-form-grid">
            <label className="settings-switch-row">
              <span>
                <strong>XXMI für {game.name}</strong>
                <small>{game.xxmiImporter} beim Start verwenden.{game.modPolicy === 'restricted' ? ' Dieses Spiel ist in GachaHub als eingeschränkt markiert; die Aktivierung erfolgt bewusst manuell.' : ''}</small>
              </span>
              <input
                type="checkbox"
                checked={effective.xxmiEnabled}
                disabled={!xxmiStatus?.installed || !(xxmiStatus.importers.find((entry) => entry.id === game.xxmiImporter)?.installed ?? false)}
                onChange={(event) => patch({ xxmiEnabled: event.target.checked })}
              />
            </label>
            <label className="settings-field">
              <span>Model Importer</span>
              <input value={game.xxmiImporter} readOnly />
            </label>
            <label className="settings-field">
              <span>XXMI Status</span>
              <input value={xxmiStatus?.installed ? `Installiert · ${xxmiStatus.version ?? 'Version unbekannt'}` : 'Nicht installiert'} readOnly />
            </label>
            <label className="settings-field settings-field-wide">
              <span>XXMI Launcher</span>
              <input value={xxmiStatus?.launcherPath ?? ''} readOnly placeholder="GachaHub Managed XXMI noch nicht installiert" />
            </label>
            {preset?.linux.xxmiLaunchArguments && (
              <div className="xxmi-requirement-card settings-field-wide">
                <div>
                  <strong><Gauge size={14} /> Zusätzliche Startbedingung</strong>
                  <code>{preset.linux.xxmiLaunchArguments}</code>
                </div>
                <button className="secondary-btn" onClick={applyXxmiRequirements}>XXMI-Argumente anwenden</button>
              </div>
            )}
            {preset?.linux.xxmiNotes?.map((note) => (
              <div className="inheritance-notice settings-field-wide" key={note}><Info size={15}/><span>{note}</span></div>
            ))}
            <div className="xxmi-test-row settings-field-wide">
              {!xxmiStatus?.installed ? (
                <button
                  className="recommended-preset-btn"
                  disabled={xxmiManagerBusy}
                  onClick={async () => {
                    setXxmiMessage('');
                    const installed = await installManagedXxmi(global);
                    setXxmiMessage(installed?.installed ? 'XXMI wurde installiert. Öffne jetzt das Importer-Setup.' : 'XXMI Installation fehlgeschlagen.');
                  }}
                >
                  <Sparkles size={14} /> {xxmiManagerBusy ? 'Installiere…' : 'XXMI installieren'}
                </button>
              ) : (
                <>
                  <button
                    className="secondary-btn"
                    disabled={xxmiBusy || xxmiManagerBusy}
                    onClick={async () => {
                      setXxmiBusy(true);
                      setXxmiMessage('');
                      try {
                        const importerReady = xxmiStatus.importers.find((entry) => entry.id === game.xxmiImporter)?.installed ?? false;
                        await launchManagedXxmi(global, game.xxmiImporter!, importerReady ? global.xxmiNoGui : false);
                        setXxmiMessage(importerReady
                          ? `${game.xxmiImporter} wurde über XXMI gestartet.`
                          : `XXMI Setup für ${game.xxmiImporter} geöffnet. Installiere den Importer dort einmalig.`);
                        await refreshManagedXxmi(global);
                      } catch (error) {
                        setXxmiMessage(String(error));
                      } finally {
                        setXxmiBusy(false);
                      }
                    }}
                  >
                    <Play size={14} /> {xxmiBusy ? 'Starte…' : ((xxmiStatus.importers.find((entry) => entry.id === game.xxmiImporter)?.installed ?? false) ? 'XXMI testen / starten' : `${game.xxmiImporter} Setup öffnen`)}
                  </button>
                </>
              )}
              {xxmiMessage && <span className="xxmi-inline-message">{xxmiMessage}</span>}
            </div>
          </div>
        ) : (
          <div className="inheritance-notice"><Info size={17}/><span>Für dieses Spiel ist aktuell kein XXMI Model Importer hinterlegt. Der Mod Manager läuft daher nur als Library-Verwaltung, bis ein eigener Adapter vorhanden ist.</span></div>
        )}
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('settings.mods')}</h3>
          <p>{preset?.modding.note ?? 'GMM-kompatible Library und optionaler Active-Mods-Ordner.'}</p>
        </div>
        <div className="mod-backend-banner">
          {modBackend === 'xxmi' ? <ShieldCheck size={17} /> : <Info size={17} />}
          <div>
            <strong>{preset?.modding.label ?? 'Mod Library'}</strong>
            <small>{deploymentAvailable ? 'Import, Profile und XXMI-Deployment vorgesehen.' : 'Import, Suche und Profile verfügbar; automatisches Loader-Deployment noch nicht aktiviert.'}</small>
          </div>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>{t('settings.modLibrary')}</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.modLibraryPath ?? '' : effective.modLibraryPath}
              placeholder={effective.modLibraryPath}
              onChange={(event) => patch({ modLibraryPath: event.target.value })}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>{t('settings.activeModsPath')}</span>
            <input
              disabled={!overrideEnabled || !deploymentAvailable}
              value={overrideEnabled ? storedGame?.activeModsPath ?? '' : effective.activeModsPath}
              placeholder={deploymentAvailable ? 'z. B. /path/to/XXMI/Mods' : 'Noch kein Deployment-Adapter für dieses Spiel'}
              onChange={(event) => patch({ activeModsPath: event.target.value })}
            />
          </label>
          <label className="settings-field">
            <span>{t('settings.deployment')}</span>
            <select
              disabled={!overrideEnabled || !deploymentAvailable}
              value={overrideEnabled ? storedGame?.modDeploymentMode ?? global.defaultModDeploymentMode : effective.modDeploymentMode}
              onChange={(event) => patch({ modDeploymentMode: event.target.value as ModDeploymentMode })}
            >
              <option value="copy">Copy (sicherer Standard)</option>
              <option value="symlink">Symlink</option>
            </select>
          </label>
          <label className="settings-field">
            <span>Legacy GMM Config</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.legacyGmmConfigPath ?? '' : effective.legacyGmmConfigPath}
              placeholder="config.json"
              onChange={(event) => patch({ legacyGmmConfigPath: event.target.value })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>{t('global.updatesMods')}</h3>
          <p>Diese Optionen gelten nur für {game.name}, wenn Overrides aktiv sind.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>{t('global.updateChannel')}</span>
            <select
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.updateChannel ?? global.updateChannel : effective.updateChannel}
              onChange={(event) => patch({ updateChannel: event.target.value as GameSettings['updateChannel'] })}
            >
              <option value="stable">Stable</option>
              <option value="beta">Beta</option>
            </select>
          </label>
          <label className="settings-field">
            <span>{t('settings.modProfile')}</span>
            <input
              disabled={!overrideEnabled || game.modPolicy === 'disabled'}
              value={overrideEnabled ? storedGame?.modProfile ?? '' : effective.modProfile}
              placeholder={effective.modProfile}
              onChange={(event) => patch({ modProfile: event.target.value })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('global.autoUpdates')}</strong><small>Updates dieses Spiels automatisch vorbereiten.</small></span>
            <input
              type="checkbox"
              disabled={!overrideEnabled}
              checked={overrideEnabled ? storedGame?.autoUpdate ?? global.autoUpdate : effective.autoUpdate}
              onChange={(event) => patch({ autoUpdate: event.target.checked })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>{t('global.preloads')}</strong><small>Vorabdownloads nutzen, sofern der Provider sie anbietet.</small></span>
            <input
              type="checkbox"
              disabled={!overrideEnabled}
              checked={overrideEnabled ? storedGame?.allowPreloads ?? global.allowPreloads : effective.allowPreloads}
              onChange={(event) => patch({ allowPreloads: event.target.checked })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled || !deploymentAvailable || game.modPolicy !== 'supported' ? 'disabled' : ''}`}>
            <span>
              <strong>Mods automatisch deployen</strong>
              <small>
                {deploymentAvailable && game.modPolicy === 'supported'
                  ? 'Mod-Profil vor dem Start über den hinterlegten Loader anwenden.'
                  : 'Für dieses Spiel ist aktuell kein automatisches Deployment freigegeben.'}
              </small>
            </span>
            <input
              type="checkbox"
              disabled={!overrideEnabled || !deploymentAvailable || game.modPolicy !== 'supported'}
              checked={
                deploymentAvailable && game.modPolicy === 'supported' &&
                (overrideEnabled ? storedGame?.modsEnabled ?? global.modsEnabledByDefault : effective.modsEnabled)
              }
              onChange={(event) => patch({ modsEnabled: event.target.checked })}
            />
          </label>
        </div>
      </section>

      <section className="effective-settings-card effective-settings-v7">
        <span>Effektive Konfiguration</span>
        <code>{effective.runtime} · {effective.runnerVersion || 'default'} · {effective.installPath}</code>
        {storedGame?.compatibilityPresetAppliedAt && (
          <small><CheckCircle2 size={12} /> Preset vom {storedGame.compatibilityPresetAppliedAt} angewendet</small>
        )}
      </section>
    </div>
  );
}
