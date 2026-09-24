import {
  CheckCircle2,
  DownloadCloud,
  RefreshCw,
  ShieldCheck,
  Trash2,
  Wrench,
} from 'lucide-react';
import { useEffect } from 'react';
import { useI18n } from '../../i18n';
import type { GlobalSettings } from '../../lib/settings';
import { useSettingsStore } from '../../stores/useSettingsStore';
import { useXxmiStore } from '../../stores/useXxmiStore';

function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  return `${(bytes / Math.pow(1024, index)).toFixed(index >= 3 ? 1 : 0)} ${units[index]}`;
}

export function XxmiManagerCard() {
  const { t } = useI18n();
  const global = useSettingsStore((state) => state.global);
  const updateGlobal = useSettingsStore((state) => state.updateGlobal);
  const { status, progress, busy, error, refresh, install, repair, remove } = useXxmiStore();

  const update = <K extends keyof GlobalSettings>(key: K, value: GlobalSettings[K]) => {
    updateGlobal(key, value);
  };

  useEffect(() => {
    void refresh(global);
  }, [global.xxmiInstallPath, global.xxmiLauncherPath, global.xxmiWineExecutable, global.xxmiWinePrefix, refresh]);

  const removeManaged = async () => {
    if (!status?.managed) return;
    if (!window.confirm('Die von GachaHub verwaltete XXMI-Installation wirklich entfernen? XXMI-Dateien in diesem Managed-Ordner werden gelöscht.')) return;
    await remove(global);
  };

  const percent = Math.round((progress?.progress ?? 0) * 100);
  const installLabel = status?.updateAvailable ? t('xxmi.updateInstall') : t('xxmi.install');

  return (
    <section className="settings-section xxmi-manager-section">
      <div className="settings-section-title compatibility-title-row">
        <div>
          <h3>{t('xxmi.title')}</h3>
          <p>{t('xxmi.description')}</p>
        </div>
        <button className="secondary-btn" onClick={() => void refresh(global)} disabled={busy}>
          <RefreshCw size={14} /> {t('xxmi.refresh')}
        </button>
      </div>

      <div className="xxmi-manager-status-grid">
        <div className={`xxmi-manager-status-card ${status?.installed ? 'good' : 'neutral'}`}>
          <span>{t('xxmi.status')}</span>
          <strong>{status?.installed ? t('xxmi.installed') : t('xxmi.notInstalled')}</strong>
          <small>{status?.managed ? t('xxmi.managed') : status?.usingCustomPath ? t('xxmi.customLegacy') : t('xxmi.readyInstall')}</small>
        </div>
        <div className="xxmi-manager-status-card neutral">
          <span>{t('xxmi.version').toUpperCase()}</span>
          <strong>{status?.version ?? '—'}</strong>
          <small>{t('xxmi.latest')}: {status?.latestVersion ?? t('xxmi.latestUnknown')}</small>
        </div>
        <div className={`xxmi-manager-status-card ${status?.updateAvailable ? 'warn' : 'good'}`}>
          <span>{t('xxmi.updateLabel')}</span>
          <strong>{status?.updateAvailable ? t('xxmi.available') : t('xxmi.current')}</strong>
          <small>{status?.latestRelease?.assetName ?? 'Portable ZIP'}</small>
        </div>
      </div>

      <div className="settings-form-grid">
        <label className="settings-field settings-field-wide">
          <span>{t('xxmi.managedPath')}</span>
          <input
            value={global.xxmiInstallPath}
            placeholder={status?.installPath || t('xxmi.managedPathPlaceholder')}
            onChange={(event) => update('xxmiInstallPath', event.target.value)}
          />
          <small>{t('xxmi.managedPathHint')}</small>
        </label>
        <label className="settings-field settings-field-wide">
          <span>{t('xxmi.customPath')}</span>
          <input
            value={global.xxmiLauncherPath}
            placeholder={t('xxmi.customPathPlaceholder')}
            onChange={(event) => update('xxmiLauncherPath', event.target.value)}
          />
          <small>{t('xxmi.customPathHint')}</small>
        </label>
        <label className="settings-field">
          <span>{t('xxmi.wine')}</span>
          <input
            value={global.xxmiWineExecutable}
            placeholder="wine"
            onChange={(event) => update('xxmiWineExecutable', event.target.value)}
          />
        </label>
        <label className="settings-field">
          <span>{t('xxmi.prefix')}</span>
          <input
            value={global.xxmiWinePrefix}
            placeholder="~/.local/share/gachahub/prefixes/xxmi"
            onChange={(event) => update('xxmiWinePrefix', event.target.value)}
          />
        </label>
        <label className="settings-switch-row">
          <span><strong>{t('xxmi.quickLaunch')}</strong><small>{t('xxmi.quickLaunchHint')}</small></span>
          <input type="checkbox" checked={global.xxmiNoGui} onChange={(event) => update('xxmiNoGui', event.target.checked)} />
        </label>
        <label className="settings-switch-row">
          <span><strong>{t('xxmi.autoUpdate')}</strong><small>{t('xxmi.autoUpdateHint')}</small></span>
          <input type="checkbox" checked={global.xxmiAutoUpdate} onChange={(event) => update('xxmiAutoUpdate', event.target.checked)} />
        </label>
      </div>

      {progress && !['ready', 'error'].includes(progress.phase) && (
        <div className="xxmi-manager-progress">
          <div><strong>{progress.message}</strong><span>{percent}%</span></div>
          <div className="hero-progress"><i style={{ width: `${percent}%` }} /></div>
          <small>{formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}</small>
        </div>
      )}

      <div className="xxmi-manager-actions">
        {(!status?.installed || status?.updateAvailable) && !status?.usingCustomPath && (
          <button className="recommended-preset-btn" disabled={busy} onClick={() => void install(global)}>
            <DownloadCloud size={15} /> {busy ? t('xxmi.wait') : installLabel}
          </button>
        )}
        {status?.installed && status.managed && (
          <button className="secondary-btn" disabled={busy} onClick={() => void repair(global)}>
            <Wrench size={14} /> {t('common.repair')}
          </button>
        )}
        {status?.installed && status.managed && (
          <button className="secondary-btn danger-soft" disabled={busy} onClick={() => void removeManaged()}>
            <Trash2 size={14} /> {t('common.remove')}
          </button>
        )}
      </div>

      {status?.launcherPath && (
        <div className="inheritance-notice">
          <ShieldCheck size={16} />
          <span><strong>{t('xxmi.launcher')}:</strong> {status.launcherPath}</span>
        </div>
      )}

      {status?.importers?.length ? (
        <div className="xxmi-importer-grid">
          {status.importers.map((importer) => (
            <div key={importer.id} className={importer.installed ? 'ready' : ''}>
              <CheckCircle2 size={14} />
              <span><strong>{importer.id}</strong><small>{importer.installed ? t('xxmi.importerDetected') : t('xxmi.importerSetupRequired')}</small></span>
            </div>
          ))}
        </div>
      ) : null}

      {status?.warnings?.map((warning) => (
        <div className="inheritance-notice" key={warning}>{warning}</div>
      ))}
      {error && <div className="mod-error">{error}</div>}
    </section>
  );
}
