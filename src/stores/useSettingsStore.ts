import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import {
  defaultGlobalSettings,
  type GameSettings,
  type GlobalSettings,
} from '../lib/settings';

interface SettingsState {
  global: GlobalSettings;
  games: Record<string, GameSettings>;
  updateGlobal: <K extends keyof GlobalSettings>(key: K, value: GlobalSettings[K]) => void;
  setGameOverride: (gameId: string, enabled: boolean) => void;
  patchGame: (gameId: string, patch: Partial<GameSettings>) => void;
  resetGame: (gameId: string) => void;
  resetGlobal: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      global: defaultGlobalSettings,
      games: {},

      updateGlobal: (key, value) =>
        set((state) => ({
          global: {
            ...state.global,
            [key]: value,
          },
        })),

      setGameOverride: (gameId, enabled) =>
        set((state) => ({
          games: {
            ...state.games,
            [gameId]: {
              ...(state.games[gameId] ?? { gameId }),
              gameId,
              overrideEnabled: enabled,
            },
          },
        })),

      patchGame: (gameId, patch) =>
        set((state) => ({
          games: {
            ...state.games,
            [gameId]: {
              ...(state.games[gameId] ?? { gameId, overrideEnabled: false }),
              ...patch,
              gameId,
            },
          },
        })),

      resetGame: (gameId) =>
        set((state) => {
          const games = { ...state.games };
          delete games[gameId];
          return { games };
        }),

      resetGlobal: () => set({ global: defaultGlobalSettings }),
    }),
    {
      name: 'gachahub-settings-v1',
      version: 4,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ global: state.global, games: state.games }),
      merge: (persistedState, currentState) => {
        const persisted = (persistedState ?? {}) as Partial<SettingsState>;
        return {
          ...currentState,
          ...persisted,
          global: {
            ...defaultGlobalSettings,
            ...(persisted.global ?? {}),
          },
          games: persisted.games ?? {},
        };
      },
    },
  ),
);
