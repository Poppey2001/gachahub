import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type { Game, InstallJob } from '../lib/types';
import { fallbackGames } from '../data/fallbackGames';

export type LauncherSection = 'game' | 'downloads' | 'mods' | 'runners' | 'settings';

interface LauncherState {
  games: Game[];
  selectedGameId: string;
  railGameIds: string[];
  jobs: InstallJob[];
  section: LauncherSection;
  requestedSettingsGameId?: string;
  catalogOpen: boolean;
  setGames: (games: Game[]) => void;
  selectGame: (id: string) => void;
  addRailGame: (id: string) => void;
  removeRailGame: (id: string) => void;
  moveRailGame: (id: string, direction: -1 | 1) => void;
  setCatalogOpen: (open: boolean) => void;
  setSection: (section: LauncherSection) => void;
  openGameSettings: (gameId: string) => void;
  clearRequestedSettingsGame: () => void;
  upsertJob: (job: InstallJob) => void;
}

const defaultRailGameIds = ['genshin'];

function sanitizeRailIds(ids: string[], games: Game[]): string[] {
  const supported = new Set(games.map((game) => game.id));
  return [...new Set(ids)].filter((id) => supported.has(id));
}

export const useLauncherStore = create<LauncherState>()(
  persist(
    (set) => ({
      games: fallbackGames,
      selectedGameId: defaultRailGameIds[0],
      railGameIds: defaultRailGameIds,
      jobs: [],
      section: 'game',
      requestedSettingsGameId: undefined,
      catalogOpen: false,

      setGames: (games) => set((state) => {
        const sanitized = sanitizeRailIds(state.railGameIds, games);
        const railGameIds = sanitized.length ? sanitized : defaultRailGameIds.filter((id) => games.some((game) => game.id === id));
        const fallbackSelected = railGameIds[0] ?? games[0]?.id ?? '';
        return {
          games,
          railGameIds,
          selectedGameId: games.some((game) => game.id === state.selectedGameId)
            ? state.selectedGameId
            : fallbackSelected,
        };
      }),

      selectGame: (id) => set((state) => ({
        selectedGameId: id,
        section: 'game',
        railGameIds: state.railGameIds.includes(id) ? state.railGameIds : [...state.railGameIds, id],
      })),

      addRailGame: (id) => set((state) => ({
        railGameIds: state.railGameIds.includes(id) ? state.railGameIds : [...state.railGameIds, id],
        selectedGameId: id,
        section: 'game',
      })),

      removeRailGame: (id) => set((state) => {
        if (state.railGameIds.length <= 1) return state;
        const railGameIds = state.railGameIds.filter((entry) => entry !== id);
        const selectedGameId = state.selectedGameId === id
          ? (railGameIds[0] ?? '')
          : state.selectedGameId;
        return { railGameIds, selectedGameId };
      }),

      moveRailGame: (id, direction) => set((state) => {
        const index = state.railGameIds.indexOf(id);
        const target = index + direction;
        if (index < 0 || target < 0 || target >= state.railGameIds.length) return state;
        const railGameIds = [...state.railGameIds];
        [railGameIds[index], railGameIds[target]] = [railGameIds[target], railGameIds[index]];
        return { railGameIds };
      }),

      setCatalogOpen: (catalogOpen) => set({ catalogOpen }),
      setSection: (section) => set({ section }),
      openGameSettings: (gameId) => set({ section: 'settings', requestedSettingsGameId: gameId }),
      clearRequestedSettingsGame: () => set({ requestedSettingsGameId: undefined }),
      upsertJob: (job) => set((state) => ({
        jobs: [...state.jobs.filter((x) => x.id !== job.id), job],
      })),
    }),
    {
      name: 'gachahub-library-v1',
      version: 1,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ railGameIds: state.railGameIds }),
      merge: (persistedState, currentState) => {
        const persisted = (persistedState ?? {}) as Partial<LauncherState>;
        return {
          ...currentState,
          railGameIds: persisted.railGameIds?.length ? persisted.railGameIds : currentState.railGameIds,
        };
      },
    },
  ),
);
