export type ModStatus = 'enabled' | 'disabled' | 'conflict' | 'broken';
export type ModDeploymentMode = 'copy' | 'symlink';

export interface ModEntry {
  id: string;
  name: string;
  relativePath: string;
  sourcePath: string;
  activePath: string;
  enabled: boolean;
  managed: boolean;
  status: ModStatus;
  iniCount: number;
  sizeBytes: number;
  conflictReason?: string;
}

export interface ModStats {
  total: number;
  enabled: number;
  disabled: number;
  conflicts: number;
  broken: number;
  managed: number;
}

export interface ModSnapshot {
  gameId: string;
  libraryPath: string;
  activePath: string;
  entries: ModEntry[];
  stats: ModStats;
}

export interface LegacyGmmConfig {
  configPath: string;
  libraryPath?: string;
  activeModsPath?: string;
  source: string;
}
