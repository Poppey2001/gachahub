export type ModPolicy = 'supported' | 'restricted' | 'disabled';
export type GameStatus = 'ready' | 'not_installed' | 'updating' | 'unsupported';

export interface GameAssets {
  background?: string;
  video?: string;
  logo?: string;
  icon?: string;
}

export interface Game {
  id: string;
  name: string;
  shortName: string;
  subtitle: string;
  publisher: string;
  provider: string;
  platforms: string[];
  installModes: string[];
  status: GameStatus;
  modPolicy: ModPolicy;
  accent: string;
  accent2: string;
  version?: string;
  notes?: string;
  xxmiImporter?: string | null;
  assets?: GameAssets;
}

export interface InstallJob {
  id: string;
  gameId: string;
  phase: 'resolving' | 'queued' | 'downloading' | 'verifying' | 'extracting' | 'downloaded' | 'ready' | 'cancelled' | 'error';
  progress: number;
  downloadedBytes: number;
  totalBytes: number;
  speedBytes: number;
  version?: string | null;
  providerMode?: string | null;
  destination?: string | null;
  message?: string | null;
}


export interface DownloadPackage {
  url: string;
  destination: string;
  size: number;
  hash?: string | null;
}

export interface DownloadPlan {
  gameId: string;
  provider: string;
  version: string;
  mode: string;
  packages: DownloadPackage[];
  totalBytes: number;
  resourceListUrl?: string | null;
  notes: string[];
}

export interface RunnerInfo {
  id: string;
  name: string;
  family: string;
  kind: string;
  version: string;
  path: string;
  source: string;
  installed: boolean;
  managedByUmu: boolean;
  removable: boolean;
  recommendedBackend: string;
  notes: string;
}

export interface RunnerCatalogEntry {
  id: string;
  name: string;
  family: string;
  kind: string;
  source: string;
  recommendedBackend: string;
  description: string;
}

export interface RunnerFamily {
  id: string;
  name: string;
  shortName: string;
  provider: string;
  source: string;
  installable: boolean;
  recommendedBackend: string;
  description: string;
}

export interface RunnerVersion {
  id: string;
  familyId: string;
  name: string;
  tagName: string;
  assetName: string;
  downloadUrl: string;
  size: number;
  publishedAt: string;
  prerelease: boolean;
  installed: boolean;
  installedPath: string;
}
