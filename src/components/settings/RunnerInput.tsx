import { Gauge, RefreshCw } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useI18n } from '../../i18n';
import type { RuntimeKind } from '../../lib/settings';
import { detectRunners } from '../../lib/tauri';
import type { RunnerInfo } from '../../lib/types';
import { useLauncherStore } from '../../stores/useLauncherStore';

interface Props {
  value: string;
  pathValue?: string;
  onChange: (value: string, path: string, runtime: RuntimeKind) => void;
  disabled?: boolean;
  placeholder?: string;
  inputId: string;
}


function runtimeForRunner(runner: RunnerInfo): RuntimeKind {
  if (runner.kind === 'native') return 'native';
  if (runner.kind.includes('wine') && !runner.kind.includes('proton')) return 'wine';
  if (runner.kind.includes('proton')) return 'umu';
  return 'auto';
}

function labelFor(runner: RunnerInfo): string {
  const version = runner.version || runner.name;
  const source = runner.source ? ` · ${runner.source}` : '';
  return `${version}${source}`;
}

export function RunnerInput({ value, pathValue = '', onChange, disabled, placeholder, inputId }: Props) {
  const { t } = useI18n();
  const setSection = useLauncherStore((state) => state.setSection);
  const [detected, setDetected] = useState<RunnerInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    setLoading(true);
    try {
      setDetected(await detectRunners());
    } catch {
      setDetected([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { void refresh(); }, []);

  const selectedRunner = useMemo(() => {
    if (pathValue) {
      const byPath = detected.find((runner) => runner.path === pathValue);
      if (byPath) return byPath;
    }
    if (value) {
      return detected.find((runner) => runner.version === value || runner.name === value);
    }
    return undefined;
  }, [detected, pathValue, value]);

  const currentValue = selectedRunner?.id ?? (value || pathValue ? '__current__' : '__auto__');

  const handleChange = (selectedId: string) => {
    if (selectedId === '__auto__') {
      onChange('', '', 'auto');
      return;
    }
    if (selectedId === '__current__') return;
    const runner = detected.find((entry) => entry.id === selectedId);
    if (!runner) return;

    // UMU's own managed runtime is intentionally left without PROTONPATH.
    // For all concrete Proton/Wine builds we persist the detected path so launch
    // does not rely on a name lookup in Steam's compatibilitytools.d.
    if (runner.managedByUmu && runner.kind === 'umu-proton') {
      onChange('', '', 'auto');
      return;
    }
    onChange(runner.version || runner.name, runner.path || '', runtimeForRunner(runner));
  };

  return (
    <div className="runner-input-control">
      <div className="runner-input-row">
        <select
          id={inputId}
          disabled={disabled || loading}
          value={currentValue}
          onChange={(event) => handleChange(event.target.value)}
          title={pathValue || placeholder}
        >
          <option value="__auto__">{placeholder ?? t('runnerPicker.auto')}</option>
          {currentValue === '__current__' && (
            <option value="__current__">{value || pathValue}</option>
          )}
          {detected.map((runner) => (
            <option key={runner.id} value={runner.id}>{labelFor(runner)}</option>
          ))}
        </select>
        <button
          type="button"
          className="runner-input-refresh"
          disabled={disabled || loading}
          onClick={() => void refresh()}
          title={t('runnerPicker.refresh')}
        >
          <RefreshCw size={14} className={loading ? 'spin' : ''} />
        </button>
        <button
          type="button"
          className="runner-input-manage"
          disabled={disabled}
          onClick={() => setSection('runners')}
          title={t('runnerPicker.manage')}
        >
          <Gauge size={14} /> {t('runnerPicker.manage')}
        </button>
      </div>
      {pathValue && <small className="runner-input-path" title={pathValue}>{pathValue}</small>}
    </div>
  );
}
