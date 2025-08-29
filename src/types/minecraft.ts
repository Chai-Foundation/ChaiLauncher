export interface MinecraftInstance {
  id: string;
  name: string;
  version: string;
  modpack?: string;
  modpackVersion?: string;
  gameDir: string;
  javaPath?: string;
  jvmArgs?: string[];
  lastPlayed?: Date;
  totalPlayTime: number;
  icon?: string;
  isModded: boolean;
  modsCount: number;
  isExternal?: boolean;
  externalLauncher?: 'gdlauncher' | 'multimc' | 'prism' | 'modrinth';
  createdAt?: string;
  sizeMb?: number;
  description?: string;
  tags?: string[];
  status?: 'ready' | 'installing' | 'failed' | 'invalid';
  installProgress?: number;
  errorMessage?: string;
}

export interface MinecraftVersion {
  id: string;
  type: 'release' | 'snapshot' | 'beta' | 'alpha';
  releaseTime: string;
  url: string;
}

export interface ModpackInfo {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  minecraftVersion: string;
  downloadUrl: string;
  iconUrl?: string;
  websiteUrl?: string;
}

// Enhanced modpack types matching Rust backend
export interface ModrinthPack {
  project_id: string;
  version_id: string;
  name: string;
  description: string;
  author: string;
  game_versions: string[];
  loaders: string[];
  downloads: number;
  icon_url?: string;
  website_url?: string;
}

export interface ModrinthVersion {
  id: string;
  project_id: string;
  author_id: string;
  featured: boolean;
  name: string;
  version_number: string;
  changelog?: string;
  changelog_url?: string;
  date_published: string;
  downloads: number;
  version_type: string;
  status: string;
  requested_status?: string;
  files: ModrinthFile[];
  dependencies: ModrinthDependency[];
  game_versions: string[];
  loaders: string[];
}

export interface ModrinthFile {
  hashes: Record<string, string>;
  url: string;
  filename: string;
  primary: boolean;
  size: number;
  file_type?: string;
}

export interface ModrinthDependency {
  version_id?: string;
  project_id?: string;
  file_name?: string;
  dependency_type: string;
}

export interface ModpackInstallProgress {
  instance_dir: string;
  progress: number;
  stage: string;
}

export interface ModpackCreationRequest {
  instanceId: string;
  instancePath: string;
  metadata: ModpackMetadata;
}

export interface ModpackMetadata {
  name: string;
  version: string;
  author: string;
  description: string;
  minecraftVersion: string;
  tags: string[];
  iconPath?: string;
  includeUserData: boolean;
  includeResourcePacks: boolean;
  includeShaderPacks: boolean;
  includeConfig: boolean;
  includeSaves: boolean;
}

export interface ModpackCreationProgress {
  instanceId: string;
  progress: number;
  stage: string;
}

export interface LauncherSettings {
  default_java_path?: string;
  default_memory: number;
  default_jvm_args: string[];
  instances_dir: string;
  downloads_dir: string;
  theme: string;
  background_image?: string;
  primary_base_color?: string;
  secondary_base_color?: string;
  auto_update: boolean;
  keepLauncherOpen?: boolean;
  showSnapshots?: boolean;
  javaPath?: string;
  maxMemory?: number;
  minMemory?: number;
  jvmArgs?: string[];
  gameDir?: string;
  auth_token?: string;
}

export interface NewsItem {
  id: string;
  title: string;
  summary: string;
  content: string;
  imageUrl?: string;
  publishedAt: string;
  category: 'minecraft' | 'launcher' | 'modding';
  url?: string;
}

export interface DownloadProgress {
  id: string;
  name: string;
  progress: number;
  status: 'downloading' | 'installing' | 'completed' | 'failed';
  totalSize?: number;
  downloadedSize?: number;
}

export interface InstallProgressEvent {
  instanceId: string;
  stage: string;
  progress: number;
  currentFile?: string;
  bytesDownloaded: number;
  totalBytes: number;
}

export interface InstallCompleteEvent {
  instanceId: string;
  success: boolean;
  error?: string;
}

