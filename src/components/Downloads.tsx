import { CheckCircle2, DownloadCloud, GripVertical, Pause, Play, X, XCircle } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useI18n } from '../i18n';
import { cancelDownload, pauseDownload } from '../lib/tauri';
import type { InstallJob } from '../lib/types';
import { useLauncherStore } from '../stores/useLauncherStore';

function fmt(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  return `${(bytes / Math.pow(1024, i)).toFixed(i >= 3 ? 1 : 0)} ${units[i]}`;
}

function isDone(job: InstallJob) {
  return ['downloaded', 'ready', 'cancelled', 'error'].includes(job.phase);
}

export function Downloads() {
  const { t } = useI18n();
  const { jobs, games } = useLauncherStore();
  const [paused, setPaused] = useState<Record<string, boolean>>({});
  const [actionError, setActionError] = useState('');
  const activeJobs = useMemo(() => jobs.filter((job) => !isDone(job)), [jobs]);
  const completedJobs = useMemo(() => jobs.filter(isDone), [jobs]);

  const togglePause = async (job: InstallJob) => {
    const next = !paused[job.id];
    setActionError('');
    try {
      await pauseDownload(job.id, next);
      setPaused((state) => ({ ...state, [job.id]: next }));
    } catch (error) {
      setActionError(error instanceof Error ? error.message : String(error));
    }
  };

  const cancel = async (job: InstallJob) => {
    setActionError('');
    try {
      await cancelDownload(job.id);
    } catch (error) {
      setActionError(error instanceof Error ? error.message : String(error));
    }
  };

  const renderJob = (job: InstallJob, finished = false) => {
    const game = games.find((entry) => entry.id === job.gameId);
    const progress = Math.round(job.progress * 100);
    const isPaused = paused[job.id] === true;
    return (
      <article className="download-row" key={job.id} style={{ '--download-accent': game?.accent ?? '#7bcfff' } as React.CSSProperties}>
        <GripVertical className="drag-handle" size={19} />
        <div className="download-avatar">{game?.shortName ?? 'GH'}</div>
        <div className="download-main">
          <div className="download-title">
            <strong>{game?.name ?? job.gameId}</strong>
            <div className="download-title-right">
              {job.version && <span className="download-version-badge">v{job.version}</span>}
              <span className={`download-phase-${job.phase}`}>{isPaused ? 'paused' : job.phase}</span>
            </div>
          </div>
          <div className="download-progress"><i style={{ width: `${progress}%` }} /></div>
          <div className="download-meta">
            <span>{fmt(job.downloadedBytes)} / {fmt(job.totalBytes)}</span>
            <span>{finished ? job.providerMode ?? 'provider' : `${fmt(job.speedBytes)}/s`}</span>
            <span>{progress}%</span>
            {job.destination && <span>{job.destination}</span>}
          </div>
          {job.message && <div className="download-message">{job.message}</div>}
        </div>
        <div className="download-actions">
          {!finished && <button onClick={() => void togglePause(job)} title={isPaused ? t('common.resume') : t('common.pause')}>{isPaused ? <Play size={16}/> : <Pause size={16}/>}</button>}
          {!finished && <button onClick={() => void cancel(job)} title={t('common.cancel')}><X size={16}/></button>}
          {finished && (job.phase === 'error' || job.phase === 'cancelled' ? <XCircle size={20}/> : <CheckCircle2 size={20}/>)}
        </div>
      </article>
    );
  };

  return (
    <main className="page-shell">
      <header className="page-header">
        <div><span className="page-eyebrow">{t('downloads.eyebrow')}</span><h1>{t('downloads.title')}</h1><p>{t('downloads.subtitle')}</p></div>
        <div className="page-header-icon"><DownloadCloud size={25} /></div>
      </header>

      {actionError && <div className="runner-error">{actionError}</div>}

      <div className="section-label"><span>{t('downloads.active')}</span><b>{activeJobs.length}</b></div>
      <section className="download-list">
        {activeJobs.length === 0 ? (
          <div className="empty-state"><DownloadCloud size={32}/><strong>{t('downloads.noneActive')}</strong><span>{t('downloads.noneActiveHint')}</span></div>
        ) : activeJobs.map((job) => renderJob(job))}
      </section>

      <div className="section-label section-spaced"><span>{t('downloads.completed')}</span><b>{completedJobs.length}</b></div>
      <section className="download-list">
        {completedJobs.length === 0 ? (
          <div className="completed-row"><CheckCircle2 size={18}/><span>{t('downloads.noneCompleted')}</span></div>
        ) : completedJobs.map((job) => renderJob(job, true))}
      </section>
    </main>
  );
}
