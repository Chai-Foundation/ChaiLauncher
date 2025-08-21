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

export interface LauncherSettings {
  javaPath: string;
  maxMemory: number;
  minMemory: number;
  jvmArgs: string[];
  gameDir: string;
  keepLauncherOpen: boolean;
  showSnapshots: boolean;
  theme: 'light' | 'dark';
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

export interface LauncherSettings {
  defaultJavaPath?: string;
  defaultMemory: number;
  defaultJvmArgs: string[];
  instancesDir: string;
  downloadsDir: string;
  theme: string;
  autoUpdate: boolean;
}