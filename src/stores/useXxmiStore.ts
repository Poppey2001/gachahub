import { create } from 'zustand';
import type { GlobalSettings } from '../lib/settings';
import {
  getManagedXxmiStatus,
  installManagedXxmi,
  removeManagedXxmi,
  repairManagedXxmi,
  startManagedXxmi,
  type XxmiManagerStatus,
  type XxmiProgress,
} from '../lib/tauri';

interface XxmiState {
  status?: XxmiManagerStatus;
  progress?: XxmiProgress;
  busy: boolean;
  error: string;
  setProgress: (progress?: XxmiProgress) => void;
  refresh: (settings: GlobalSettings) => Promise<XxmiManagerStatus | undefined>;
  install: (settings: GlobalSettings) => Promise<XxmiManagerStatus | undefined>;
  repair: (settings: GlobalSettings) => Promise<XxmiManagerStatus | undefined>;
  remove: (settings: GlobalSettings) => Promise<void>;
  launch: (settings: GlobalSettings, importer: string, noGui?: boolean) => Promise<void>;
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useXxmiStore = create<XxmiState>((set) => ({
  busy: false,
  error: '',

  setProgress: (progress) => set({ progress }),

  refresh: async (settings) => {
    try {
      const status = await getManagedXxmiStatus(
        settings.xxmiInstallPath,
        settings.xxmiLauncherPath,
        settings.xxmiWineExecutable,
        settings.xxmiWinePrefix,
      );
      set({ status, error: '' });
      return status;
    } catch (error) {
      set({ error: message(error) });
      return undefined;
    }
  },

  install: async (settings) => {
    set({ busy: true, error: '' });
    try {
      const status = await installManagedXxmi(settings.xxmiInstallPath);
      set({ status, busy: false });
      return status;
    } catch (error) {
      set({ busy: false, error: message(error) });
      return undefined;
    }
  },

  repair: async (settings) => {
    set({ busy: true, error: '' });
    try {
      const status = await repairManagedXxmi(settings.xxmiInstallPath);
      set({ status, busy: false });
      return status;
    } catch (error) {
      set({ busy: false, error: message(error) });
      return undefined;
    }
  },

  remove: async (settings) => {
    set({ busy: true, error: '' });
    try {
      await removeManagedXxmi(settings.xxmiInstallPath);
      const status = await getManagedXxmiStatus(
        settings.xxmiInstallPath,
        settings.xxmiLauncherPath,
        settings.xxmiWineExecutable,
        settings.xxmiWinePrefix,
      );
      set({ status, busy: false, progress: undefined });
    } catch (error) {
      set({ busy: false, error: message(error) });
    }
  },

  launch: async (settings, importer, noGui = settings.xxmiNoGui) => {
    set({ busy: true, error: '' });
    try {
      await startManagedXxmi(
        settings.xxmiInstallPath,
        settings.xxmiLauncherPath,
        importer,
        settings.xxmiWineExecutable,
        settings.xxmiWinePrefix,
        noGui,
      );
      set({ busy: false });
    } catch (error) {
      set({ busy: false, error: message(error) });
      throw error;
    }
  },
}));
