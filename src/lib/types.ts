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
  phase: 'queued' | 'downloading' | 'verifying' | 'extracting' | 'ready' | 'error';
  progress: number;
  downloadedBytes: number;
  totalBytes: number;
  speedBytes: number;
}
