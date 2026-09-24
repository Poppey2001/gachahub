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
  applyDetectedGameDefaults: (gameId: string, patch: Partial<GameSettings>) => void;
  resetGame: (gameId: string) => void;
  resetGlobal: () => void;
}

function migrateGames(games: Record<string, GameSettings> | undefined, forceEnable = false): Record<string, GameSettings> {
  if (!games) return {};
  return Object.fromEntries(
    Object.entries(games).map(([gameId, game]) => [
      gameId,
      {
        ...game,
        gameId,
        // v0.10 migration: game-specific settings are enabled by default.
        overrideEnabled: forceEnable ? true : (game.overrideEnabled ?? true),
      },
    ]),
  );
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
              ...(state.games[gameId] ?? { gameId, overrideEnabled: true }),
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
              ...(state.games[gameId] ?? { gameId, overrideEnabled: true }),
              ...patch,
              gameId,
            },
          },
        })),

      applyDetectedGameDefaults: (gameId, patch) =>
        set((state) => {
          const current = state.games[gameId] ?? { gameId, overrideEnabled: true };
          const next: GameSettings = { ...current, gameId, overrideEnabled: true };
          const alwaysRefresh = new Set<keyof GameSettings>([
            'postInstallProfile',
            'postInstallAppliedAt',
            'compatibilityPresetAppliedAt',
          ]);

          for (const [rawKey, value] of Object.entries(patch)) {
            const key = rawKey as keyof GameSettings;
            if (key === 'gameId' || key === 'overrideEnabled' || value === undefined) continue;
            if (alwaysRefresh.has(key) || current[key] === undefined) {
              (next as unknown as Record<string, unknown>)[key] = value;
            }
          }

          return {
            games: {
              ...state.games,
              [gameId]: next,
            },
          };
        }),

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
      version: 6,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ global: state.global, games: state.games }),
      migrate: (persistedState, persistedVersion) => {
        const persisted = (persistedState ?? {}) as Partial<SettingsState>;
        return {
          global: {
            ...defaultGlobalSettings,
            ...(persisted.global ?? {}),
          },
          games: migrateGames(persisted.games, persistedVersion < 5),
        };
      },
      merge: (persistedState, currentState) => {
        const persisted = (persistedState ?? {}) as Partial<SettingsState>;
        return {
          ...currentState,
          ...persisted,
          global: {
            ...defaultGlobalSettings,
            ...(persisted.global ?? {}),
          },
          games: migrateGames(persisted.games),
        };
      },
    },
  ),
);
