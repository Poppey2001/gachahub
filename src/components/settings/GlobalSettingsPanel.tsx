import { RotateCcw } from 'lucide-react';
import { useI18n } from '../../i18n';
import type { GlobalSettings, RuntimeKind } from '../../lib/settings';
import { useSettingsStore } from '../../stores/useSettingsStore';
import { RunnerInput } from './RunnerInput';
import { XxmiManagerCard } from './XxmiManagerCard';

interface GlobalSettingsPanelProps { platform: string; }

function numberOrNull(value: string): number | null {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.max(0, parsed) : null;
}

export function GlobalSettingsPanel({ platform }: GlobalSettingsPanelProps) {
  const { t } = useI18n();
  const global = useSettingsStore((state) => state.global);
  const updateGlobal = useSettingsStore((state) => state.updateGlobal);
  const resetGlobal = useSettingsStore((state) => state.resetGlobal);
  const update = <K extends keyof GlobalSettings>(key: K, value: GlobalSettings[K]) => updateGlobal(key, value);

  return (
    <div className="settings-panel-stack">
      <div className="settings-panel-header">
        <div><span className="eyebrow">{t('global.eyebrow')}</span><h2>{t('global.title')}</h2><p>{t('global.subtitle')}</p></div>
        <button className="secondary-btn" onClick={resetGlobal}><RotateCcw size={15} /> {t('global.defaults')}</button>
      </div>

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.general')}</h3><p>{t('global.generalHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-field"><span>{t('global.detectedSystem')}</span><input value={platform} readOnly /></label>
          <label className="settings-field">
            <span>{t('global.language')}</span>
            <select value={global.language} onChange={(event) => update('language', event.target.value as GlobalSettings['language'])}>
              <option value="de">{t('language.de')}</option><option value="en">{t('language.en')}</option><option value="fr">{t('language.fr')}</option><option value="es">{t('language.es')}</option>
            </select>
          </label>
          <label className="settings-field">
            <span>{t('global.theme')}</span>
            <select value={global.theme} onChange={(event) => update('theme', event.target.value as GlobalSettings['theme'])}>
              <option value="dark">{t('theme.dark')}</option><option value="light">{t('theme.light')}</option><option value="system">{t('theme.system')}</option>
            </select>
          </label>
          <label className="settings-switch-row"><span><strong>{t('global.debug')}</strong><small>{t('global.debugHint')}</small></span><input type="checkbox" checked={global.debugLogging} onChange={(event) => update('debugLogging', event.target.checked)} /></label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.appearance')}</h3><p>{t('global.appearanceHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-switch-row"><span><strong>{t('global.animated')}</strong><small>{t('global.animatedHint')}</small></span><input type="checkbox" checked={global.animatedBackgrounds} onChange={(event) => update('animatedBackgrounds', event.target.checked)} /></label>
          <label className="settings-field"><span>{t('global.dim', { value: global.backgroundDim })}</span><input type="range" min={20} max={85} step={1} value={global.backgroundDim} onChange={(event) => update('backgroundDim', Number(event.target.value))} /></label>
          <label className="settings-field settings-field-wide"><span>{t('global.blur', { value: global.backgroundBlur })}</span><input type="range" min={0} max={20} step={1} value={global.backgroundBlur} onChange={(event) => update('backgroundBlur', Number(event.target.value))} /></label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.downloadInstall')}</h3><p>{t('global.downloadInstallHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-field settings-field-wide"><span>{t('global.downloadDir')}</span><input value={global.downloadPath} onChange={(event) => update('downloadPath', event.target.value)} /></label>
          <label className="settings-field settings-field-wide"><span>{t('global.installDir')}</span><input value={global.defaultInstallPath} onChange={(event) => update('defaultInstallPath', event.target.value)} /></label>
          <label className="settings-field"><span>{t('global.parallel')}</span><input type="number" min={1} max={32} value={global.parallelDownloads} onChange={(event) => update('parallelDownloads', Math.min(32, Math.max(1, Number(event.target.value) || 1)))} /></label>
          <label className="settings-field"><span>{t('global.bandwidth')}</span><input type="number" min={0} placeholder={t('common.unlimited')} value={global.bandwidthLimitMbps ?? ''} onChange={(event) => update('bandwidthLimitMbps', numberOrNull(event.target.value))} /></label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.runtime')}</h3><p>{t('global.runtimeHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-field"><span>{t('global.defaultRuntime')}</span><select value={global.defaultRuntime} onChange={(event) => update('defaultRuntime', event.target.value as RuntimeKind)}><option value="auto">Auto</option><option value="native">Native</option><option value="umu">umu</option><option value="ge-proton">GE-Proton</option><option value="proton">Proton</option><option value="wine">Wine</option></select></label>
          <div className="settings-field"><span>{t('global.defaultRunner')}</span><RunnerInput inputId="global-runner" value={global.defaultRunnerVersion} pathValue={global.defaultRunnerPath} onChange={(value, path, runtime) => { update('defaultRunnerVersion', value); update('defaultRunnerPath', path); update('defaultRuntime', runtime); }} placeholder={t('global.runnerPlaceholder')} /></div>
          <label className="settings-field settings-field-wide"><span>{t('global.prefixBase')}</span><input value={global.prefixBasePath} onChange={(event) => update('prefixBasePath', event.target.value)} /></label>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.extras')}</h3><p>{t('global.extrasHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-switch-row"><span><strong>{t('global.gamescope')}</strong><small>{t('global.gamescopeHint')}</small></span><input type="checkbox" checked={global.defaultGamescopeEnabled} onChange={(event) => update('defaultGamescopeEnabled', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.gamemode')}</strong><small>{t('global.gamemodeHint')}</small></span><input type="checkbox" checked={global.defaultGameModeEnabled} onChange={(event) => update('defaultGameModeEnabled', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.mangohud')}</strong><small>{t('global.mangohudHint')}</small></span><input type="checkbox" checked={global.defaultMangoHudEnabled} onChange={(event) => update('defaultMangoHudEnabled', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.preventSleep')}</strong><small>{t('global.preventSleepHint')}</small></span><input type="checkbox" checked={global.defaultPreventSleep} onChange={(event) => update('defaultPreventSleep', event.target.checked)} /></label>
          <label className="settings-field"><span>{t('global.fpsLimit')}</span><input type="number" min={0} placeholder={t('common.unlimited')} value={global.defaultFpsLimit ?? ''} onChange={(event) => update('defaultFpsLimit', numberOrNull(event.target.value))} /></label>
        </div>
      </section>

      <XxmiManagerCard />

      <section className="settings-section">
        <div className="settings-section-title"><h3>{t('global.updatesMods')}</h3><p>{t('global.updatesModsHint')}</p></div>
        <div className="settings-form-grid">
          <label className="settings-field"><span>{t('global.updateChannel')}</span><select value={global.updateChannel} onChange={(event) => update('updateChannel', event.target.value as GlobalSettings['updateChannel'])}><option value="stable">{t('channel.stable')}</option><option value="beta">{t('channel.beta')}</option></select></label>
          <label className="settings-field"><span>{t('global.modProfile')}</span><input value={global.defaultModProfile} onChange={(event) => update('defaultModProfile', event.target.value)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.autoUpdates')}</strong><small>{t('global.autoUpdatesHint')}</small></span><input type="checkbox" checked={global.autoUpdate} onChange={(event) => update('autoUpdate', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.preloads')}</strong><small>{t('global.preloadsHint')}</small></span><input type="checkbox" checked={global.allowPreloads} onChange={(event) => update('allowPreloads', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.modsDefault')}</strong><small>{t('global.modsDefaultHint')}</small></span><input type="checkbox" checked={global.modsEnabledByDefault} onChange={(event) => update('modsEnabledByDefault', event.target.checked)} /></label>
          <label className="settings-switch-row"><span><strong>{t('global.verify')}</strong><small>{t('global.verifyHint')}</small></span><input type="checkbox" checked={global.verifyBeforeLaunch} onChange={(event) => update('verifyBeforeLaunch', event.target.checked)} /></label>
        </div>
      </section>
    </div>
  );
}
