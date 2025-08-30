import { invoke } from '@tauri-apps/api/core';
import { LauncherSettings } from '../types/minecraft';
import { applyColorScheme } from '../utils/colors';

export class SettingsService {
  static async loadSettings(): Promise<LauncherSettings> {
    try {
      return await invoke('get_launcher_settings') as LauncherSettings;
    } catch (error) {
      console.error('Failed to load settings:', error);
      return this.getDefaultSettings();
    }
  }

  static async updateSettings(settings: LauncherSettings): Promise<void> {
    await invoke('update_launcher_settings', { settings });
    applyColorScheme(settings);
  }

  static getDefaultSettings(): LauncherSettings {
    return {
      default_memory: 4096,
      default_jvm_args: ['-XX:+UnlockExperimentalVMOptions', '-XX:+UseG1GC'],
      instances_dir: '/minecraft/instances',
      downloads_dir: '/minecraft/downloads',
      theme: 'dark',
      primary_base_color: '#78716c',
      secondary_base_color: '#eb9109',
      auto_update: true,
      keepLauncherOpen: true,
      showSnapshots: false,
      javaPath: '',
      maxMemory: 4096,
      minMemory: 1024,
      jvmArgs: ['-XX:+UnlockExperimentalVMOptions', '-XX:+UseG1GC'],
      gameDir: '/minecraft',
    };
  }

  static async openFolder(path: string): Promise<void> {
    await invoke('open_folder', { path });
  }
}

export default SettingsService;