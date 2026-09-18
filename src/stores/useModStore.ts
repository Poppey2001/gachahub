import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

interface ModStoreState {
  activeProfiles: Record<string, string>;
  profiles: Record<string, Record<string, string[]>>;
  setActiveProfile: (gameId: string, profile: string) => void;
  saveProfile: (gameId: string, profile: string, enabledIds: string[]) => void;
  deleteProfile: (gameId: string, profile: string) => void;
}

export const useModStore = create<ModStoreState>()(
  persist(
    (set) => ({
      activeProfiles: {},
      profiles: {},
      setActiveProfile: (gameId, profile) =>
        set((state) => ({ activeProfiles: { ...state.activeProfiles, [gameId]: profile } })),
      saveProfile: (gameId, profile, enabledIds) =>
        set((state) => ({
          profiles: {
            ...state.profiles,
            [gameId]: {
              ...(state.profiles[gameId] ?? {}),
              [profile]: Array.from(new Set(enabledIds)),
            },
          },
        })),
      deleteProfile: (gameId, profile) =>
        set((state) => {
          const gameProfiles = { ...(state.profiles[gameId] ?? {}) };
          delete gameProfiles[profile];
          return { profiles: { ...state.profiles, [gameId]: gameProfiles } };
        }),
    }),
    { name: 'gachahub-mod-profiles-v1', storage: createJSONStorage(() => localStorage) },
  ),
);
