import { RotateCcw } from 'lucide-react';
import type { GlobalSettings, RuntimeKind } from '../../lib/settings';
import { useSettingsStore } from '../../stores/useSettingsStore';

interface GlobalSettingsPanelProps {
  platform: string;
}

function numberOrNull(value: string): number | null {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.max(0, parsed) : null;
}

export function GlobalSettingsPanel({ platform }: GlobalSettingsPanelProps) {
  const global = useSettingsStore((state) => state.global);
  const updateGlobal = useSettingsStore((state) => state.updateGlobal);
  const resetGlobal = useSettingsStore((state) => state.resetGlobal);

  const update = <K extends keyof GlobalSettings>(key: K, value: GlobalSettings[K]) => {
    updateGlobal(key, value);
  };

  return (
    <div className="settings-panel-stack">
      <div className="settings-panel-header">
        <div>
          <span className="eyebrow">GLOBAL SETTINGS</span>
          <h2>Globale Einstellungen</h2>
          <p>Diese Werte werden standardmäßig von jedem Spiel geerbt.</p>
        </div>
        <button className="secondary-btn" onClick={resetGlobal}>
          <RotateCcw size={15} /> Standardwerte
        </button>
      </div>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Allgemein</h3>
          <p>Oberfläche und erkannte Plattform.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>Erkanntes System</span>
            <input value={platform} readOnly />
          </label>
          <label className="settings-field">
            <span>Sprache</span>
            <select
              value={global.language}
              onChange={(event) => update('language', event.target.value as GlobalSettings['language'])}
            >
              <option value="de">Deutsch</option>
              <option value="en">English</option>
            </select>
          </label>
          <label className="settings-field">
            <span>Theme</span>
            <select
              value={global.theme}
              onChange={(event) => update('theme', event.target.value as GlobalSettings['theme'])}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="system">System</option>
            </select>
          </label>
          <label className="settings-switch-row">
            <span>
              <strong>Debug Logging</strong>
              <small>Zusätzliche Logs für Provider, Downloads und Runtime.</small>
            </span>
            <input
              type="checkbox"
              checked={global.debugLogging}
              onChange={(event) => update('debugLogging', event.target.checked)}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Launcher-Optik</h3>
          <p>Twintail-artige Hero-Hintergründe und Bewegungen konfigurieren.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-switch-row">
            <span>
              <strong>Animierte Hintergründe</strong>
              <small>MP4-Hintergründe verwenden, wenn ein Game-Asset vorhanden ist.</small>
            </span>
            <input
              type="checkbox"
              checked={global.animatedBackgrounds}
              onChange={(event) => update('animatedBackgrounds', event.target.checked)}
            />
          </label>
          <label className="settings-field">
            <span>Hintergrund abdunkeln ({global.backgroundDim}%)</span>
            <input
              type="range"
              min={20}
              max={85}
              step={1}
              value={global.backgroundDim}
              onChange={(event) => update('backgroundDim', Number(event.target.value))}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Artwork Blur ({global.backgroundBlur}px)</span>
            <input
              type="range"
              min={0}
              max={20}
              step={1}
              value={global.backgroundBlur}
              onChange={(event) => update('backgroundBlur', Number(event.target.value))}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Downloads & Installation</h3>
          <p>Standardpfade und Download-Limits.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>Download-Verzeichnis</span>
            <input
              value={global.downloadPath}
              onChange={(event) => update('downloadPath', event.target.value)}
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Standard-Installationspfad</span>
            <input
              value={global.defaultInstallPath}
              onChange={(event) => update('defaultInstallPath', event.target.value)}
            />
          </label>
          <label className="settings-field">
            <span>Parallele Downloads</span>
            <input
              type="number"
              min={1}
              max={32}
              value={global.parallelDownloads}
              onChange={(event) =>
                update('parallelDownloads', Math.min(32, Math.max(1, Number(event.target.value) || 1)))
              }
            />
          </label>
          <label className="settings-field">
            <span>Bandbreitenlimit (Mbit/s)</span>
            <input
              type="number"
              min={0}
              placeholder="Unbegrenzt"
              value={global.bandwidthLimitMbps ?? ''}
              onChange={(event) => update('bandwidthLimitMbps', numberOrNull(event.target.value))}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Linux Runtime</h3>
          <p>Standardwerte für Wine, Proton, GE-Proton und umu.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>Standard Runtime</span>
            <select
              value={global.defaultRuntime}
              onChange={(event) => update('defaultRuntime', event.target.value as RuntimeKind)}
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
            <span>Standard Runner</span>
            <input
              value={global.defaultRunnerVersion}
              onChange={(event) => update('defaultRunnerVersion', event.target.value)}
              placeholder="z. B. GE-Proton 11"
            />
          </label>
          <label className="settings-field settings-field-wide">
            <span>Prefix-Basisverzeichnis</span>
            <input
              value={global.prefixBasePath}
              onChange={(event) => update('prefixBasePath', event.target.value)}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Linux Extras</h3>
          <p>Standardwerte für Gamescope, GameMode, MangoHud und Session-Verhalten.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-switch-row">
            <span><strong>Gamescope standardmäßig</strong><small>Für Spiele verwenden, die von einem isolierten Compositor profitieren.</small></span>
            <input type="checkbox" checked={global.defaultGamescopeEnabled} onChange={(event) => update('defaultGamescopeEnabled', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>GameMode standardmäßig</strong><small>gamemoderun verwenden, sobald der Launch-Backend angeschlossen ist.</small></span>
            <input type="checkbox" checked={global.defaultGameModeEnabled} onChange={(event) => update('defaultGameModeEnabled', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>MangoHud standardmäßig</strong><small>Performance-Overlay für unterstützte Starts aktivieren.</small></span>
            <input type="checkbox" checked={global.defaultMangoHudEnabled} onChange={(event) => update('defaultMangoHudEnabled', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>System-Wakeup verhindern</strong><small>Während des Spielens Suspend/Idle verhindern.</small></span>
            <input type="checkbox" checked={global.defaultPreventSleep} onChange={(event) => update('defaultPreventSleep', event.target.checked)} />
          </label>
          <label className="settings-field">
            <span>Standard FPS-Limit</span>
            <input
              type="number"
              min={0}
              placeholder="Unbegrenzt"
              value={global.defaultFpsLimit ?? ''}
              onChange={(event) => update('defaultFpsLimit', numberOrNull(event.target.value))}
            />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>XXMI Launcher</h3>
          <p>Gemeinsamer XXMI-Pfad für GIMI, SRMI, ZZMI, WWMI und EFMI.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide">
            <span>XXMI Launcher.exe</span>
            <input
              value={global.xxmiLauncherPath}
              placeholder="z. B. ~/Games/XXMI Launcher/Resources/Bin/XXMI Launcher.exe"
              onChange={(event) => update('xxmiLauncherPath', event.target.value)}
            />
          </label>
          <label className="settings-field">
            <span>Linux Wine Executable</span>
            <input
              value={global.xxmiWineExecutable}
              placeholder="wine"
              onChange={(event) => update('xxmiWineExecutable', event.target.value)}
            />
          </label>
          <label className="settings-switch-row">
            <span><strong>Quick Launch / No GUI</strong><small>Startet XXMI mit --nogui --xxmi &lt;Importer&gt;.</small></span>
            <input type="checkbox" checked={global.xxmiNoGui} onChange={(event) => update('xxmiNoGui', event.target.checked)} />
          </label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title">
          <h3>Updates & Mods</h3>
          <p>Globale Standardrichtlinien für Spiele.</p>
        </div>
        <div className="settings-form-grid">
          <label className="settings-field">
            <span>Update-Kanal</span>
            <select
              value={global.updateChannel}
              onChange={(event) => update('updateChannel', event.target.value as GlobalSettings['updateChannel'])}
            >
              <option value="stable">Stable</option>
              <option value="beta">Beta</option>
            </select>
          </label>
          <label className="settings-field">
            <span>Standard Mod-Profil</span>
            <input
              value={global.defaultModProfile}
              onChange={(event) => update('defaultModProfile', event.target.value)}
            />
          </label>
          <label className="settings-switch-row">
            <span><strong>Automatische Updates</strong><small>Spiele standardmäßig automatisch aktualisieren.</small></span>
            <input type="checkbox" checked={global.autoUpdate} onChange={(event) => update('autoUpdate', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>Preloads erlauben</strong><small>Verfügbare Preload-Pakete vorab laden.</small></span>
            <input type="checkbox" checked={global.allowPreloads} onChange={(event) => update('allowPreloads', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>Mods standardmäßig aktiv</strong><small>Nur wenn der Game-Adapter Modding zulässt.</small></span>
            <input type="checkbox" checked={global.modsEnabledByDefault} onChange={(event) => update('modsEnabledByDefault', event.target.checked)} />
          </label>
          <label className="settings-switch-row">
            <span><strong>Vor Start verifizieren</strong><small>Dateien vor jedem Spielstart prüfen.</small></span>
            <input type="checkbox" checked={global.verifyBeforeLaunch} onChange={(event) => update('verifyBeforeLaunch', event.target.checked)} />
          </label>
        </div>
      </section>
    </div>
  );
}
