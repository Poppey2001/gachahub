import { Check, CloudDownload, Gauge, LoaderCircle, RefreshCw, Search, Star, Trash2 } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useI18n } from '../i18n';
import { detectRunners, getRunnerFamilies, getRunnerVersions, installRunnerVersion, removeRunnerVersion } from '../lib/tauri';
import type { RunnerFamily, RunnerInfo, RunnerVersion } from '../lib/types';
import type { RuntimeKind } from '../lib/settings';
import { useSettingsStore } from '../stores/useSettingsStore';

function runtimeFor(kind: string): RuntimeKind {
  if (kind === 'native') return 'native';
  if (kind.includes('wine') && !kind.includes('proton')) return 'wine';
  if (kind.includes('proton')) return 'umu';
  return 'auto';
}

function formatBytes(size: number): string {
  if (!size) return '';
  const gb = size / 1024 / 1024 / 1024;
  if (gb >= 1) return `${gb.toFixed(gb >= 10 ? 0 : 1)} GB`;
  return `${Math.round(size / 1024 / 1024)} MB`;
}

function shortVersion(version: RunnerVersion) {
  return version.tagName || version.name;
}

export function Runners() {
  const { t } = useI18n();
  const [families, setFamilies] = useState<RunnerFamily[]>([]);
  const [selectedFamily, setSelectedFamily] = useState('ge-proton');
  const [versions, setVersions] = useState<RunnerVersion[]>([]);
  const [detected, setDetected] = useState<RunnerInfo[]>([]);
  const [loadingFamilies, setLoadingFamilies] = useState(true);
  const [loadingVersions, setLoadingVersions] = useState(false);
  const [busyVersion, setBusyVersion] = useState('');
  const [search, setSearch] = useState('');
  const [error, setError] = useState('');
  const global = useSettingsStore((state) => state.global);
  const updateGlobal = useSettingsStore((state) => state.updateGlobal);

  const loadFamilies = async () => {
    setLoadingFamilies(true);
    setError('');
    try {
      const [familyList, local] = await Promise.all([getRunnerFamilies(), detectRunners()]);
      setFamilies(familyList);
      setDetected(local);
      if (!familyList.some((f) => f.id === selectedFamily) && familyList[0]) setSelectedFamily(familyList[0].id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoadingFamilies(false);
    }
  };

  const loadVersions = async (familyId = selectedFamily) => {
    if (!familyId) return;
    setLoadingVersions(true);
    setError('');
    try {
      const [remote, local] = await Promise.all([getRunnerVersions(familyId), detectRunners()]);
      setVersions(remote);
      setDetected(local);
    } catch (err) {
      setVersions([]);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoadingVersions(false);
    }
  };

  useEffect(() => { void loadFamilies(); }, []);
  useEffect(() => { if (selectedFamily) void loadVersions(selectedFamily); }, [selectedFamily]);

  const activeFamily = families.find((family) => family.id === selectedFamily);
  const visibleVersions = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return versions;
    return versions.filter((version) => `${version.name} ${version.tagName}`.toLowerCase().includes(q));
  }, [versions, search]);

  const installedForFamily = versions.filter((v) => v.installed).length || detected.filter((r) => {
    if (selectedFamily === 'ge-proton') return r.kind === 'ge-proton';
    if (selectedFamily === 'dwproton') return r.kind === 'dwproton';
    if (selectedFamily === 'cachyos-proton') return r.kind === 'cachyos-proton';
    if (selectedFamily === 'proton-em') return r.kind === 'proton-em';
    if (selectedFamily === 'proton-sarek') return r.kind === 'proton-sarek';
    if (selectedFamily === 'proton-wineland') return r.kind === 'proton-wineland';
    if (selectedFamily === 'steam-proton') return r.kind === 'steam-proton' || r.kind === 'proton-experimental' || r.kind === 'proton-hotfix';
    if (selectedFamily === 'umu-proton') return r.kind === 'umu-proton';
    return false;
  }).length;

  const install = async (version: RunnerVersion) => {
    setBusyVersion(version.id);
    setError('');
    try {
      const runner = await installRunnerVersion(version.familyId, version.tagName);
      setDetected(await detectRunners());
      setVersions((current) => current.map((v) => v.id === version.id ? { ...v, installed: true, installedPath: runner.path } : v));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyVersion('');
    }
  };

  const remove = async (version: RunnerVersion) => {
    if (!version.installedPath) return;
    setBusyVersion(version.id);
    setError('');
    try {
      await removeRunnerVersion(version.installedPath);
      if (global.defaultRunnerPath === version.installedPath) {
        updateGlobal('defaultRunnerVersion', '');
        updateGlobal('defaultRunnerPath', '');
        updateGlobal('defaultRuntime', 'auto');
      }
      setDetected(await detectRunners());
      setVersions((current) => current.map((v) => v.id === version.id ? { ...v, installed: false, installedPath: '' } : v));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyVersion('');
    }
  };

  const setDefault = (version: RunnerVersion) => {
    const local = detected.find((runner) => runner.path === version.installedPath || runner.version === version.tagName || runner.name === version.tagName);
    updateGlobal('defaultRuntime', runtimeFor(local?.kind ?? selectedFamily));
    if (local?.managedByUmu && local.kind === 'umu-proton') {
      updateGlobal('defaultRunnerVersion', '');
      updateGlobal('defaultRunnerPath', '');
    } else {
      updateGlobal('defaultRunnerVersion', local?.version || version.tagName);
      updateGlobal('defaultRunnerPath', local?.path || version.installedPath || '');
    }
  };

  return (
    <main className="runner-manager-page">
      <header className="runner-manager-header">
        <div className="runner-manager-title-icon"><Gauge size={26}/></div>
        <div className="runner-manager-heading">
          <h1>{t('runnerManager.title')}</h1>
          <p>{t('runnerManager.count', { installed: installedForFamily, available: versions.length })}</p>
        </div>
        <button className="runner-manager-refresh" onClick={() => { void loadFamilies(); void loadVersions(); }} disabled={loadingFamilies || loadingVersions}>
          <RefreshCw size={16}/>{t('runnerManager.refresh')}
        </button>
      </header>

      {error && <div className="runner-manager-error">{error}</div>}

      <section className="runner-manager-shell">
        <aside className="runner-family-list">
          {loadingFamilies && <div className="runner-family-loading"><LoaderCircle className="spin" size={20}/></div>}
          {families.map((family) => (
            <button key={family.id} className={`runner-family-item ${family.id === selectedFamily ? 'active' : ''}`} onClick={() => setSelectedFamily(family.id)}>
              <span className="runner-family-orbit">◎</span>
              <div><strong>{family.name}</strong><small>{family.source}</small></div>
              {family.id === selectedFamily && <span className="runner-family-accent"/>}
            </button>
          ))}
        </aside>

        <div className="runner-version-pane">
          <div className="runner-version-toolbar">
            <div>
              <strong>{activeFamily?.name ?? t('runnerManager.title')}</strong>
              <span>{activeFamily?.description}</span>
            </div>
            <label className="runner-version-search"><Search size={15}/><input value={search} onChange={(e) => setSearch(e.target.value)} placeholder={t('runnerManager.search')}/></label>
          </div>

          <div className="runner-version-list">
            {loadingVersions && <div className="runner-version-empty"><LoaderCircle className="spin" size={24}/><span>{t('runnerManager.loading')}</span></div>}
            {!loadingVersions && visibleVersions.length === 0 && <div className="runner-version-empty"><Gauge size={25}/><span>{activeFamily?.installable === false ? t('runnerManager.externalManaged') : t('runnerManager.none')}</span></div>}
            {!loadingVersions && visibleVersions.map((version) => {
              const busy = busyVersion === version.id;
              const installedRunner = detected.find((runner) => runner.path === version.installedPath);
              const canRemove = Boolean(version.installedPath && installedRunner?.removable);
              const isDefault = Boolean(version.installedPath && global.defaultRunnerPath === version.installedPath)
                || (!global.defaultRunnerPath && (global.defaultRunnerVersion === version.tagName || global.defaultRunnerVersion === version.name));
              return (
                <article className={`runner-version-row ${version.installed ? 'installed' : ''}`} key={version.id}>
                  <span className="runner-version-dot"/>
                  <div className="runner-version-name">
                    <strong>{shortVersion(version)}</strong>
                    <div>
                      {version.prerelease && <span className="runner-beta-pill">{t('runnerManager.prerelease')}</span>}
                      {version.installed && <span className="runner-installed-pill"><Check size={11}/>{t('runnerManager.installed')}</span>}
                      {isDefault && <span className="runner-default-pill"><Star size={10}/>{t('runnerManager.default')}</span>}
                      {!!version.size && <small>{formatBytes(version.size)}</small>}
                    </div>
                  </div>
                  <div className="runner-version-actions">
                    {version.installed ? (
                      <>
                        <button className="runner-version-secondary" onClick={() => setDefault(version)} disabled={busy || isDefault}><Star size={14}/>{isDefault ? t('runnerManager.default') : t('runnerManager.makeDefault')}</button>
                        {canRemove && <button className="runner-version-remove" onClick={() => void remove(version)} disabled={busy}><Trash2 size={14}/>{busy ? t('runnerManager.working') : t('runnerManager.remove')}</button>}
                      </>
                    ) : activeFamily?.installable ? (
                      <button className="runner-version-install" onClick={() => void install(version)} disabled={busy}><CloudDownload size={15}/>{busy ? t('runnerManager.installing') : t('runnerManager.install')}</button>
                    ) : <span className="runner-external-badge">{t('runnerManager.external')}</span>}
                  </div>
                </article>
              );
            })}
          </div>
        </div>
      </section>
    </main>
  );
}
