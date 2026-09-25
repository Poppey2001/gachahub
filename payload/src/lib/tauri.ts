import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { DownloadPlan, Game, InstallJob, RunnerCatalogEntry, RunnerFamily, RunnerInfo, RunnerVersion } from './types';
import type { LegacyGmmConfig, ModDeploymentMode, ModSnapshot } from './mods';


export async function chooseDirectory(title = 'Ordner auswählen', defaultPath?: string): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title,
    defaultPath: defaultPath && !defaultPath.startsWith('~') ? defaultPath : undefined,
  });
  return typeof selected === 'string' ? selected : null;
}

export interface XxmiStatus {
  importer: string;
  launcherPath: string;
  exists: boolean;
  platform: string;
  launchCommand: string[];
}

export interface XxmiImporterStatus {
  id: string;
  installed: boolean;
  path?: string | null;
}

export interface XxmiReleaseInfo {
  version: string;
  tagName: string;
  assetName: string;
  downloadUrl: string;
  size: number;
  digest?: string | null;
  publishedAt?: string | null;
}

export interface XxmiManagerStatus {
  installed: boolean;
  valid: boolean;
  managed: boolean;
  usingCustomPath: boolean;
  version?: string | null;
  latestVersion?: string | null;
  updateAvailable: boolean;
  installPath: string;
  launcherPath: string;
  platform: string;
  importers: XxmiImporterStatus[];
  latestRelease?: XxmiReleaseInfo | null;
  warnings: string[];
}

export interface XxmiProgress {
  phase: string;
  progress: number;
  downloadedBytes: number;
  totalBytes: number;
  message: string;
}

export async function backendGames(): Promise<Game[]> {
  try {
    return await invoke<Game[]>('list_games');
  } catch {
    return [];
  }
}

export async function beginInstall(gameId: string, installPath = ''): Promise<InstallJob> {
  return invoke<InstallJob>('start_game_download', { gameId, installPath });
}

export async function resolveDownloadPlan(gameId: string): Promise<DownloadPlan> {
  return invoke<DownloadPlan>('resolve_download_plan', { gameId });
}

export async function pauseDownload(jobId: string, paused: boolean): Promise<void> {
  await invoke('pause_download', { jobId, paused });
}

export async function cancelDownload(jobId: string): Promise<void> {
  await invoke('cancel_download', { jobId });
}

export async function detectRunners(): Promise<RunnerInfo[]> {
  return invoke<RunnerInfo[]>('detect_runners');
}

export async function getRunnerCatalog(): Promise<RunnerCatalogEntry[]> {
  return invoke<RunnerCatalogEntry[]>('runner_catalog');
}

export async function getRunnerFamilies(): Promise<RunnerFamily[]> {
  return invoke<RunnerFamily[]>('runner_families');
}

export async function getRunnerVersions(familyId: string): Promise<RunnerVersion[]> {
  return invoke<RunnerVersion[]>('runner_versions', { familyId });
}

export async function installRunnerVersion(familyId: string, tagName: string): Promise<RunnerInfo> {
  return invoke<RunnerInfo>('install_runner', { familyId, tagName });
}

export async function removeRunnerVersion(path: string): Promise<void> {
  await invoke('remove_runner', { path });
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

export async function getManagedXxmiStatus(
  installPath: string,
  customLauncherPath: string,
  wineExecutable: string,
  winePrefix: string,
): Promise<XxmiManagerStatus> {
  return invoke<XxmiManagerStatus>('xxmi_manager_status', { installPath, customLauncherPath, wineExecutable, winePrefix });
}

export async function installManagedXxmi(installPath: string): Promise<XxmiManagerStatus> {
  return invoke<XxmiManagerStatus>('install_xxmi', { installPath });
}

export async function repairManagedXxmi(installPath: string): Promise<XxmiManagerStatus> {
  return invoke<XxmiManagerStatus>('repair_xxmi', { installPath });
}

export async function removeManagedXxmi(installPath: string): Promise<void> {
  await invoke('remove_xxmi', { installPath });
}

export async function startManagedXxmi(
  installPath: string,
  customLauncherPath: string,
  importer: string,
  wineExecutable: string,
  winePrefix: string,
  noGui: boolean,
): Promise<XxmiStatus> {
  return invoke<XxmiStatus>('launch_managed_xxmi', {
    installPath,
    customLauncherPath,
    importer,
    wineExecutable,
    winePrefix,
    noGui,
  });
}


export interface PostInstallDetectedTools {
  gameMode: boolean;
  gamescope: boolean;
  mangoHud: boolean;
  umu: boolean;
}

export interface PostInstallSettingsPatch {
  overrideEnabled: true;
  installPath: string;
  runtime: 'auto' | 'native' | 'wine' | 'proton' | 'ge-proton' | 'umu';
  runnerVersion: string;
  runnerPath: string;
  prefixPath: string;
  launchArguments: string;
  environmentVariables: string;
  gamescopeEnabled: boolean;
  gameModeEnabled: boolean;
  mangoHudEnabled: boolean;
  preventSleep: boolean;
  autoUpdate: boolean;
  allowPreloads: boolean;
  updateChannel: 'stable' | 'beta';
  modsEnabled: boolean;
  verifyBeforeLaunch: boolean;
  xxmiEnabled: boolean;
  postInstallProfile: string;
  postInstallAppliedAt: string;
}

export interface PostInstallResult {
  gameId: string;
  platform: string;
  executablePath: string;
  settings: PostInstallSettingsPatch;
  detected: PostInstallDetectedTools;
  warnings: string[];
}

export async function postInstallDefaults(gameId: string, installPath: string, prefixPath: string): Promise<PostInstallResult> {
  return invoke<PostInstallResult>('post_install_defaults', { gameId, installPath, prefixPath });
}

export interface LaunchGameRequest {
  gameId: string;
  executablePath: string;
  runtime: string;
  runnerVersion: string;
  runnerPath: string;
  prefixPath: string;
  launchArguments: string;
  environmentVariables: Record<string, string>;
  gamescopeEnabled: boolean;
  gameModeEnabled: boolean;
  mangoHudEnabled: boolean;
  preventSleep: boolean;
}

export interface LaunchGameResult {
  pid: number;
  gameId: string;
  runtime: string;
  executablePath: string;
  prefixPath: string;
  runnerVersion: string;
  runnerPath: string;
  commandPreview: string;
}

export async function launchGame(request: LaunchGameRequest): Promise<LaunchGameResult> {
  return invoke<LaunchGameResult>('launch_game', { ...request });
}
