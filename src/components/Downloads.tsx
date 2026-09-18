import { CheckCircle2, DownloadCloud, GripVertical, Pause, X } from 'lucide-react';
import { useLauncherStore } from '../stores/useLauncherStore';

function fmt(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i >= 3 ? 1 : 0)} ${units[i]}`;
}

export function Downloads() {
  const { jobs, games } = useLauncherStore();
  return (
    <main className="page-shell">
      <header className="page-header">
        <div><span className="page-eyebrow">TRANSFER QUEUE</span><h1>Downloads</h1><p>Installationen, Updates, Verify und Repair in einer Queue.</p></div>
        <div className="page-header-icon"><DownloadCloud size={25} /></div>
      </header>

      <div className="section-label"><span>Aktiv</span><b>{jobs.length}</b></div>
      <section className="download-list">
        {jobs.length === 0 ? (
          <div className="empty-state"><DownloadCloud size={32}/><strong>Keine aktiven Downloads</strong><span>Starte eine Installation von einer Game-Seite.</span></div>
        ) : jobs.map((job) => {
          const game = games.find((entry) => entry.id === job.gameId);
          const progress = Math.round(job.progress * 100);
          return (
            <article className="download-row" key={job.id} style={{ '--download-accent': game?.accent ?? '#7bcfff' } as React.CSSProperties}>
              <GripVertical className="drag-handle" size={19} />
              <div className="download-avatar">{game?.shortName ?? 'GH'}</div>
              <div className="download-main">
                <div className="download-title"><strong>{game?.name ?? job.gameId}</strong><span>{job.phase}</span></div>
                <div className="download-progress"><i style={{ width: `${progress}%` }} /></div>
                <div className="download-meta"><span>{fmt(job.downloadedBytes)} / {fmt(job.totalBytes)}</span><span>{fmt(job.speedBytes)}/s</span><span>{progress}%</span></div>
              </div>
              <div className="download-actions"><button><Pause size={16}/></button><button><X size={16}/></button></div>
            </article>
          );
        })}
      </section>

      <div className="section-label section-spaced"><span>Abgeschlossen</span></div>
      <div className="completed-row"><CheckCircle2 size={18}/><span>Noch keine abgeschlossenen Jobs in dieser Session.</span></div>
    </main>
  );
}
