import { Download, Gamepad2, Library, Puzzle, Settings } from 'lucide-react';
import { useLauncherStore } from '../stores/useLauncherStore';

export function Sidebar() {
  const { section, setSection } = useLauncherStore();
  const items = [
    ['library', Library, 'Bibliothek'],
    ['downloads', Download, 'Downloads'],
    ['mods', Puzzle, 'Mods'],
    ['settings', Settings, 'Einstellungen'],
  ] as const;

  return (
    <aside className="sidebar">
      <div className="brand"><Gamepad2 size={24}/><span>GachaHub</span><b>v0.2</b></div>
      <nav>
        {items.map(([id, Icon, label]) => (
          <button key={id} className={section === id ? 'nav-active' : ''} onClick={() => setSection(id)}>
            <Icon size={18}/><span>{label}</span>
          </button>
        ))}
      </nav>
      <div className="sidebar-footer">Windows + Linux<br/><span>Provider-first architecture</span></div>
    </aside>
  );
}
