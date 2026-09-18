import { Check, Download, Gauge, Trash2 } from 'lucide-react';

const runners = [
  { family: 'GE-Proton', version: 'GE-Proton 11-6', installed: true, recommended: true },
  { family: 'GE-Proton', version: 'GE-Proton 10-26', installed: false, recommended: false },
  { family: 'Proton', version: 'Proton 10', installed: true, recommended: false },
  { family: 'Wine', version: 'Wine-GE Latest', installed: false, recommended: false },
  { family: 'Steam Runtime', version: 'SteamRT3 Sniper', installed: true, recommended: false },
];

export function Runners() {
  return (
    <main className="page-shell">
      <header className="page-header">
        <div><span className="page-eyebrow">COMPATIBILITY</span><h1>Runners</h1><p>Wine-, Proton- und Steam-Runtimes zentral verwalten.</p></div>
        <div className="page-header-icon"><Gauge size={25} /></div>
      </header>

      <section className="runner-grid">
        {runners.map((runner) => (
          <article className="runner-card" key={`${runner.family}-${runner.version}`}>
            <div className="runner-card-head">
              <div className="runner-badge"><Gauge size={20} /></div>
              <div><span>{runner.family}</span><strong>{runner.version}</strong></div>
              {runner.recommended && <em>Recommended</em>}
            </div>
            <div className="runner-status">
              {runner.installed ? <><Check size={15} /> Installiert</> : 'Verfügbar'}
            </div>
            <button className={runner.installed ? 'runner-action danger' : 'runner-action'}>
              {runner.installed ? <><Trash2 size={16} /> Entfernen</> : <><Download size={16} /> Installieren</>}
            </button>
          </article>
        ))}
      </section>
    </main>
  );
}
