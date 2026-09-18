import { create } from 'zustand';
import type { Game, InstallJob } from '../lib/types';
import { fallbackGames } from '../data/fallbackGames';

export type LauncherSection = 'game' | 'downloads' | 'mods' | 'runners' | 'settings';

interface LauncherState {
  games: Game[];
  selectedGameId: string;
  jobs: InstallJob[];
  section: LauncherSection;
  requestedSettingsGameId?: string;
  setGames: (games: Game[]) => void;
  selectGame: (id: string) => void;
  setSection: (section: LauncherSection) => void;
  openGameSettings: (gameId: string) => void;
  clearRequestedSettingsGame: () => void;
  upsertJob: (job: InstallJob) => void;
}

export const useLauncherStore = create<LauncherState>((set) => ({
  games: fallbackGames,
  selectedGameId: fallbackGames[0].id,
  jobs: [],
  section: 'game',
  requestedSettingsGameId: undefined,
  setGames: (games) => set((state) => ({
    games,
    selectedGameId: games.some((game) => game.id === state.selectedGameId)
      ? state.selectedGameId
      : games[0]?.id ?? '',
  })),
  selectGame: (id) => set({ selectedGameId: id, section: 'game' }),
  setSection: (section) => set({ section }),
  openGameSettings: (gameId) => set({ section: 'settings', requestedSettingsGameId: gameId }),
  clearRequestedSettingsGame: () => set({ requestedSettingsGameId: undefined }),
  upsertJob: (job) => set((state) => ({
    jobs: [...state.jobs.filter((x) => x.id !== job.id), job],
  })),
}));
