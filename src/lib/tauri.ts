import { invoke } from '@tauri-apps/api/core';
import type { Game, InstallJob } from './types';
import type { LegacyGmmConfig, ModDeploymentMode, ModSnapshot } from './mods';

export interface XxmiStatus {
  importer: string;
  launcherPath: string;
  exists: boolean;
  platform: string;
  launchCommand: string[];
}

export async function backendGames(): Promise<Game[]> {
  try {
    return await invoke<Game[]>('list_games');
  } catch {
    return [];
  }
}

export async function beginInstall(gameId: string): Promise<InstallJob> {
  return invoke<InstallJob>('start_install', { gameId });
}

export async function backendPlatform(): Promise<string> {
  try {
    return await invoke<string>('platform_name');
  } catch {
    return navigator.platform;
  }
}

export async function scanMods(gameId: string, libraryPath: string, activePath: string): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('scan_mods', { gameId, libraryPath, activePath });
}

export async function enableMod(gameId: string, modId: string, libraryPath: string, activePath: string, deploymentMode: ModDeploymentMode): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('enable_mod', { gameId, modId, libraryPath, activePath, deploymentMode });
}

export async function adoptMod(gameId: string, modId: string, libraryPath: string, activePath: string): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('adopt_mod', { gameId, modId, libraryPath, activePath });
}

export async function disableMod(gameId: string, modId: string, libraryPath: string, activePath: string): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('disable_mod', { gameId, modId, libraryPath, activePath });
}

export async function importModFolder(gameId: string, sourcePath: string, libraryPath: string, activePath: string): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('import_mod_folder', { gameId, sourcePath, libraryPath, activePath });
}

export async function importModSource(gameId: string, sourcePath: string, libraryPath: string, activePath: string): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('import_mod_source', { gameId, sourcePath, libraryPath, activePath });
}

export async function archiveToolStatus(): Promise<string> {
  return invoke<string>('archive_tool_status');
}

export async function syncModProfile(gameId: string, enabledIds: string[], libraryPath: string, activePath: string, deploymentMode: ModDeploymentMode): Promise<ModSnapshot> {
  return invoke<ModSnapshot>('sync_mod_profile', { gameId, enabledIds, libraryPath, activePath, deploymentMode });
}

export async function readLegacyGmmConfig(path: string): Promise<LegacyGmmConfig> {
  return invoke<LegacyGmmConfig>('read_legacy_gmm_config', { path });
}

export async function getXxmiStatus(
  launcherPath: string,
  importer: string,
  wineExecutable: string,
  noGui: boolean,
): Promise<XxmiStatus> {
  return invoke<XxmiStatus>('xxmi_status', { launcherPath, importer, wineExecutable, noGui });
}

export async function startXxmi(
  launcherPath: string,
  importer: string,
  wineExecutable: string,
  noGui: boolean,
): Promise<XxmiStatus> {
  return invoke<XxmiStatus>('launch_xxmi', { launcherPath, importer, wineExecutable, noGui });
}
