export type RuntimeKind = 'auto' | 'native' | 'wine' | 'proton' | 'ge-proton' | 'umu';
export type ModDeploymentMode = 'copy' | 'symlink';
export type UpdateChannel = 'stable' | 'beta';
export type ThemeKind = 'dark' | 'light' | 'system';
export type LanguageKind = 'de' | 'en';

export interface GlobalSettings {
  language: LanguageKind;
  theme: ThemeKind;
  animatedBackgrounds: boolean;
  backgroundDim: number;
  backgroundBlur: number;
  downloadPath: string;
  defaultInstallPath: string;
  parallelDownloads: number;
  bandwidthLimitMbps: number | null;
  defaultRuntime: RuntimeKind;
  defaultRunnerVersion: string;
  prefixBasePath: string;
  defaultGamescopeEnabled: boolean;
  defaultGameModeEnabled: boolean;
  defaultMangoHudEnabled: boolean;
  defaultPreventSleep: boolean;
  defaultFpsLimit: number | null;
  autoUpdate: boolean;
  allowPreloads: boolean;
  updateChannel: UpdateChannel;
  modsEnabledByDefault: boolean;
  defaultModProfile: string;
  modRootPath: string;
  defaultModDeploymentMode: ModDeploymentMode;
  verifyBeforeLaunch: boolean;
  debugLogging: boolean;
  xxmiLauncherPath: string;
  xxmiWineExecutable: string;
  xxmiNoGui: boolean;
}

export interface GameSettings {
  gameId: string;
  overrideEnabled: boolean;
  installPath?: string;
  runtime?: RuntimeKind;
  runnerVersion?: string;
  prefixPath?: string;
  launchArguments?: string;
  environmentVariables?: string;
  gamescopeEnabled?: boolean;
  gameModeEnabled?: boolean;
  mangoHudEnabled?: boolean;
  preventSleep?: boolean;
  fpsLimit?: number | null;
  autoUpdate?: boolean;
  allowPreloads?: boolean;
  updateChannel?: UpdateChannel;
  modsEnabled?: boolean;
  modProfile?: string;
  modLibraryPath?: string;
  activeModsPath?: string;
  modDeploymentMode?: ModDeploymentMode;
  legacyGmmConfigPath?: string;
  verifyBeforeLaunch?: boolean;
  xxmiEnabled?: boolean;
  compatibilityPresetAppliedAt?: string;
}

export interface ResolvedGameSettings {
  installPath: string;
  runtime: RuntimeKind;
  runnerVersion: string;
  prefixPath: string;
  launchArguments: string;
  environmentVariables: Record<string, string>;
  gamescopeEnabled: boolean;
  gameModeEnabled: boolean;
  mangoHudEnabled: boolean;
  preventSleep: boolean;
  fpsLimit: number | null;
  autoUpdate: boolean;
  allowPreloads: boolean;
  updateChannel: UpdateChannel;
  modsEnabled: boolean;
  modProfile: string;
  modLibraryPath: string;
  activeModsPath: string;
  modDeploymentMode: ModDeploymentMode;
  legacyGmmConfigPath: string;
  verifyBeforeLaunch: boolean;
  xxmiEnabled: boolean;
  xxmiLauncherPath: string;
  xxmiWineExecutable: string;
  xxmiNoGui: boolean;
}

export const defaultGlobalSettings: GlobalSettings = {
  language: 'de',
  theme: 'dark',
  animatedBackgrounds: true,
  backgroundDim: 58,
  backgroundBlur: 0,
  downloadPath: '~/Games/GachaHub/Downloads',
  defaultInstallPath: '~/Games/GachaHub/Games',
  parallelDownloads: 8,
  bandwidthLimitMbps: null,
  defaultRuntime: 'auto',
  defaultRunnerVersion: 'GE-Proton',
  prefixBasePath: '~/.local/share/gachahub/prefixes',
  defaultGamescopeEnabled: false,
  defaultGameModeEnabled: false,
  defaultMangoHudEnabled: false,
  defaultPreventSleep: true,
  defaultFpsLimit: null,
  autoUpdate: true,
  allowPreloads: true,
  updateChannel: 'stable',
  modsEnabledByDefault: false,
  defaultModProfile: 'Default',
  modRootPath: '~/.local/share/gachahub/mods',
  defaultModDeploymentMode: 'copy',
  verifyBeforeLaunch: false,
  debugLogging: false,
  xxmiLauncherPath: '',
  xxmiWineExecutable: 'wine',
  xxmiNoGui: true,
};

export function parseEnvironmentVariables(value?: string): Record<string, string> {
  if (!value?.trim()) return {};

  return value
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#') && line.includes('='))
    .reduce<Record<string, string>>((result, line) => {
      const separator = line.indexOf('=');
      const key = line.slice(0, separator).trim();
      const entryValue = line.slice(separator + 1).trim();
      if (key) result[key] = entryValue;
      return result;
    }, {});
}

export function resolveGameSettings(
  gameId: string,
  global: GlobalSettings,
  game?: GameSettings,
): ResolvedGameSettings {
  const useOverride = game?.overrideEnabled === true;
  const gameInstallPath = `${global.defaultInstallPath}/${gameId}`;
  const gamePrefixPath = `${global.prefixBasePath}/${gameId}`;

  return {
    installPath: useOverride && game?.installPath ? game.installPath : gameInstallPath,
    runtime: useOverride && game?.runtime ? game.runtime : global.defaultRuntime,
    runnerVersion:
      useOverride && game?.runnerVersion ? game.runnerVersion : global.defaultRunnerVersion,
    prefixPath: useOverride && game?.prefixPath ? game.prefixPath : gamePrefixPath,
    launchArguments: useOverride ? game?.launchArguments ?? '' : '',
    environmentVariables: parseEnvironmentVariables(
      useOverride ? game?.environmentVariables : undefined,
    ),
    gamescopeEnabled:
      useOverride && game?.gamescopeEnabled !== undefined
        ? game.gamescopeEnabled
        : global.defaultGamescopeEnabled,
    gameModeEnabled:
      useOverride && game?.gameModeEnabled !== undefined
        ? game.gameModeEnabled
        : global.defaultGameModeEnabled,
    mangoHudEnabled:
      useOverride && game?.mangoHudEnabled !== undefined
        ? game.mangoHudEnabled
        : global.defaultMangoHudEnabled,
    preventSleep:
      useOverride && game?.preventSleep !== undefined
        ? game.preventSleep
        : global.defaultPreventSleep,
    fpsLimit:
      useOverride && game?.fpsLimit !== undefined
        ? game.fpsLimit
        : global.defaultFpsLimit,
    autoUpdate:
      useOverride && game?.autoUpdate !== undefined ? game.autoUpdate : global.autoUpdate,
    allowPreloads:
      useOverride && game?.allowPreloads !== undefined
        ? game.allowPreloads
        : global.allowPreloads,
    updateChannel:
      useOverride && game?.updateChannel ? game.updateChannel : global.updateChannel,
    modsEnabled:
      useOverride && game?.modsEnabled !== undefined
        ? game.modsEnabled
        : global.modsEnabledByDefault,
    modProfile:
      useOverride && game?.modProfile ? game.modProfile : global.defaultModProfile,
    modLibraryPath:
      useOverride && game?.modLibraryPath
        ? game.modLibraryPath
        : `${global.modRootPath}/${gameId}/library`,
    activeModsPath: useOverride ? game?.activeModsPath ?? '' : '',
    modDeploymentMode:
      useOverride && game?.modDeploymentMode
        ? game.modDeploymentMode
        : global.defaultModDeploymentMode,
    legacyGmmConfigPath: useOverride ? game?.legacyGmmConfigPath ?? '' : '',
    verifyBeforeLaunch:
      useOverride && game?.verifyBeforeLaunch !== undefined
        ? game.verifyBeforeLaunch
        : global.verifyBeforeLaunch,
    // XXMI is deliberately independent from the general override switch. It is a per-game launch toggle.
    xxmiEnabled: game?.xxmiEnabled ?? false,
    xxmiLauncherPath: global.xxmiLauncherPath,
    xxmiWineExecutable: global.xxmiWineExecutable,
    xxmiNoGui: global.xxmiNoGui,
  };
}
