// Mod management types for ChaiLauncher frontend

export interface ModInfo {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  game_versions: string[];
  loaders: string[];
  downloads: number;
  icon_url?: string;
  website_url?: string;
  source_url?: string;
  license?: string;
  categories: string[];
  side: 'Client' | 'Server' | 'Both' | 'Unknown';
  source: 'CurseForge' | 'Modrinth' | 'GitHub' | { Direct: string } | 'Local';
  featured: boolean;
  date_created: string;
  date_updated: string;
}

export interface ModFile {
  id: string;
  mod_id: string;
  filename: string;
  display_name: string;
  version: string;
  size: number;
  download_url: string;
  hashes: Record<string, string>;
  dependencies: ModDependency[];
  game_versions: string[];
  loaders: string[];
  release_type: 'Release' | 'Beta' | 'Alpha';
  date_published: string;
  primary: boolean;
}

export interface ModDependency {
  mod_id: string;
  version_id?: string;
  file_name?: string;
  dependency_type: 'Required' | 'Optional' | 'Incompatible' | 'Embedded';
}

export interface InstalledMod {
  mod_info: ModInfo;
  installed_file: ModFile;
  install_path: string;
  enabled: boolean;
  install_date: string;
  update_available?: ModFile;
}

export type ModLoader = 
  | { Forge: string }
  | { Fabric: string }
  | { Quilt: string }
  | { NeoForge: string }
  | { ModLoader: string }
  | { Rift: string };

export interface ModSearchFilters {
  query: string;
  game_version?: string;
  mod_loader?: string;
  category?: string;
  sort_by?: 'relevance' | 'downloads' | 'updated' | 'created';
  limit?: number;
}

export interface ModInstallProgress {
  instance_id: string;
  mod_id: string;
  progress: number;
  downloaded: number;
  total: number;
}

export interface ModUpdateProgress {
  instance_id: string;
  mod_id: string;
  progress: number;
  downloaded: number;
  total: number;
}

// Events
export interface ModInstalledEvent {
  instance_id: string;
  mod: InstalledMod;
}

export interface ModUninstalledEvent {
  instance_id: string;
  mod_id: string;
}

export interface ModUpdatedEvent {
  instance_id: string;
  mod_id: string;
}

export interface ModEnabledChangedEvent {
  instance_id: string;
  mod_id: string;
  enabled: boolean;
}

export interface ModUpdatesCheckedEvent {
  instance_id: string;
  mods_with_updates: string[];
}

export interface ModLoaderInstalledEvent {
  instance_id: string;
  loader: string;
  version: string;
}