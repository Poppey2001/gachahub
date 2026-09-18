import {
  AlertTriangle,
  CheckCircle2,
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
import { getCompatibilityPreset } from '../../data/compatibilityPresets';
import { startXxmi } from '../../lib/tauri';
import { useSettingsStore } from '../../stores/useSettingsStore';

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
  const global = useSettingsStore((state) => state.global);
  const storedGame = useSettingsStore((state) => state.games[game.id]);
  const setGameOverride = useSettingsStore((state) => state.setGameOverride);
  const patchGame = useSettingsStore((state) => state.patchGame);
  const resetGame = useSettingsStore((state) => state.resetGame);
  const overrideEnabled = storedGame?.overrideEnabled ?? false;
  const effective = resolveGameSettings(game.id, global, storedGame);
  const preset = getCompatibilityPreset(game.id);
  const [xxmiMessage, setXxmiMessage] = useState('');
  const [xxmiBusy, setXxmiBusy] = useState(false);

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
      runnerVersion: preset.linux.recommendedRunner,
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
              <h3>Kompatibilität</h3>
              <p>Recherche-Stand {preset.checkedAt}. Community-/Launcher-Kompatibilität, keine offizielle Linux-Zusage.</p>
            </div>
            <button className="recommended-preset-btn" onClick={applyRecommendedPreset}>
              <Sparkles size={15} /> Empfohlene Linux-Werte anwenden
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

      <section className="settings-section override-card">
        <label className="override-toggle">
          <span>
            <strong>Eigene Einstellungen für {game.name}</strong>
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
          <h3>Installation</h3>
          <p>Installationspfad und dateibezogene Optionen.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>Installationspfad</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.installPath ?? '' : effective.installPath}
              placeholder={effective.installPath}
              onChange={(event) => patch({ installPath: event.target.value })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>Vor Start verifizieren</strong><small>Integrität der Spieldateien vor dem Start prüfen.</small></span>
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
          <h3>Runtime</h3>
          <p>Game-spezifische Linux-Kompatibilitätsumgebung.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>Runtime</span>
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
          <label className="settings-field">
            <span>Runner-Version / Familie</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.runnerVersion ?? '' : effective.runnerVersion}
              placeholder={preset?.linux.recommendedRunner ?? effective.runnerVersion}
              onChange={(event) => patch({ runnerVersion: event.target.value })}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Prefix-Pfad</span>
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
          <h3>Linux Extras</h3>
          <p>Pro Spiel steuerbare Hilfen für Windowing, Performance und Session-Verhalten.</p>
        </div>
        <div className="settings-form-grid">
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>Gamescope</strong><small>Kann besonders bei Fullscreen-/Input-Problemen helfen.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.gamescopeEnabled} onChange={(event) => patch({ gamescopeEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>GameMode</strong><small>gamemoderun beim späteren Launch-Backend verwenden.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.gameModeEnabled} onChange={(event) => patch({ gameModeEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>MangoHud</strong><small>Performance-Overlay für diesen Titel.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.mangoHudEnabled} onChange={(event) => patch({ mangoHudEnabled: event.target.checked })} />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>Idle / Suspend verhindern</strong><small>System während des Spielens wach halten.</small></span>
            <input type="checkbox" disabled={!overrideEnabled} checked={effective.preventSleep} onChange={(event) => patch({ preventSleep: event.target.checked })} />
          </label>
          <label className="settings-field">
            <span>FPS-Limit</span>
            <input
              type="number"
              min={0}
              disabled={!overrideEnabled}
              placeholder="Unbegrenzt"
              value={overrideEnabled ? storedGame?.fpsLimit ?? '' : effective.fpsLimit ?? ''}
              onChange={(event) => patch({ fpsLimit: event.target.value ? Math.max(0, Number(event.target.value)) : null })}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Launch</h3>
          <p>Startargumente und Umgebungsvariablen.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>Launch Arguments</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.launchArguments ?? '' : ''}
              placeholder={preset?.linux.launchArguments || 'z. B. -fullscreen'}
              onChange={(event) => patch({ launchArguments: event.target.value })}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Environment Variables</span>
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
          <p>Game-spezifischer XXMI-Schalter. Der allgemeine Launcher-Pfad liegt in den Global Settings.</p>
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
                onChange={(event) => patch({ xxmiEnabled: event.target.checked })}
              />
            </label>
            <label className="settings-field">
              <span>Model Importer</span>
              <input value={game.xxmiImporter} readOnly />
            </label>
            <label className="settings-field settings-field-wide">
              <span>XXMI Launcher</span>
              <input value={global.xxmiLauncherPath} readOnly placeholder="Unter Global Settings konfigurieren" />
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
              <button
                className="secondary-btn"
                disabled={xxmiBusy || !global.xxmiLauncherPath.trim()}
                onClick={async () => {
                  setXxmiBusy(true);
                  setXxmiMessage('');
                  try {
                    await startXxmi(global.xxmiLauncherPath, game.xxmiImporter!, global.xxmiWineExecutable, global.xxmiNoGui);
                    setXxmiMessage(`${game.xxmiImporter} wurde über XXMI gestartet.`);
                  } catch (error) {
                    setXxmiMessage(String(error));
                  } finally {
                    setXxmiBusy(false);
                  }
                }}
              >
                <Play size={14} /> {xxmiBusy ? 'Starte…' : 'XXMI testen / starten'}
              </button>
              {xxmiMessage && <span className="xxmi-inline-message">{xxmiMessage}</span>}
            </div>
          </div>
        ) : (
          <div className="inheritance-notice"><Info size={17}/><span>Für dieses Spiel ist aktuell kein XXMI Model Importer hinterlegt. Der Mod Manager läuft daher nur als Library-Verwaltung, bis ein eigener Adapter vorhanden ist.</span></div>
        )}
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Mod Manager</h3>
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
            <span>Mod Library</span>
            <input
              disabled={!overrideEnabled}
              value={overrideEnabled ? storedGame?.modLibraryPath ?? '' : effective.modLibraryPath}
              placeholder={effective.modLibraryPath}
              onChange={(event) => patch({ modLibraryPath: event.target.value })}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Active Mods / Loader Mods</span>
            <input
              disabled={!overrideEnabled || !deploymentAvailable}
              value={overrideEnabled ? storedGame?.activeModsPath ?? '' : effective.activeModsPath}
              placeholder={deploymentAvailable ? 'z. B. /path/to/XXMI/Mods' : 'Noch kein Deployment-Adapter für dieses Spiel'}
              onChange={(event) => patch({ activeModsPath: event.target.value })}
            />
          </label>
          <label className="settings-field">
            <span>Deployment</span>
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
          <h3>Updates & Mods</h3>
          <p>Diese Optionen gelten nur für {game.name}, wenn Overrides aktiv sind.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>Update-Kanal</span>
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
            <span>Mod-Profil</span>
            <input
              disabled={!overrideEnabled || game.modPolicy === 'disabled'}
              value={overrideEnabled ? storedGame?.modProfile ?? '' : effective.modProfile}
              placeholder={effective.modProfile}
              onChange={(event) => patch({ modProfile: event.target.value })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>Automatische Updates</strong><small>Updates dieses Spiels automatisch vorbereiten.</small></span>
            <input
              type="checkbox"
              disabled={!overrideEnabled}
              checked={overrideEnabled ? storedGame?.autoUpdate ?? global.autoUpdate : effective.autoUpdate}
              onChange={(event) => patch({ autoUpdate: event.target.checked })}
            />
          </label>
          <label className={`settings-switch-row ${!overrideEnabled ? 'disabled' : ''}`}>
            <span><strong>Preloads erlauben</strong><small>Vorabdownloads nutzen, sofern der Provider sie anbietet.</small></span>
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
