import { invoke } from '@tauri-apps/api/core';
import { MinecraftInstance } from '../types/minecraft';

export class InstanceService {
  static async loadInstances(): Promise<MinecraftInstance[]> {
    try {
      const storedInstances = await invoke('load_instances') as MinecraftInstance[];
      return storedInstances.filter(instance => {
        const gameDir = (instance as any).gameDir || (instance as any).game_dir;
        return !!gameDir;
      }).map(instance => {
        const rawInstance = instance as any;
        return {
          id: rawInstance.id,
          name: rawInstance.name,
          version: rawInstance.version,
          modpack: rawInstance.modpack,
          modpackVersion: rawInstance.modpack_version,
          gameDir: rawInstance.game_dir || rawInstance.gameDir,
          javaPath: rawInstance.java_path,
          jvmArgs: rawInstance.jvm_args,
          lastPlayed: rawInstance.last_played ? new Date(rawInstance.last_played) : undefined,
          totalPlayTime: rawInstance.total_play_time || 0,
          icon: rawInstance.icon,
          isModded: rawInstance.is_modded || false,
          modsCount: rawInstance.mods_count || 0,
          isExternal: rawInstance.is_external,
          externalLauncher: rawInstance.external_launcher,
          status: rawInstance.status || 'ready' as const
        } as MinecraftInstance;
      });
    } catch (error) {
      console.error('Failed to load instances:', error);
      return [];
    }
  }

  static async detectExternalInstances(): Promise<MinecraftInstance[]> {
    try {
      const externalInstances = await invoke('detect_all_external_instances');
      return (externalInstances as any[]).map(instance => {
        const gameDir = instance.path || instance.game_dir || '';
        let status: 'ready' | 'invalid' = 'ready';
        let errorMessage: string | undefined;
        
        if (!gameDir) {
          status = 'invalid';
          errorMessage = 'Missing game directory path';
        } else if (!instance.name) {
          status = 'invalid';
          errorMessage = 'Missing instance name';
        } else if (!instance.version) {
          status = 'invalid';
          errorMessage = 'Missing Minecraft version';
        }
        
        return {
          id: instance.id,
          name: instance.name || 'Unknown Instance',
          version: instance.version || 'Unknown',
          gameDir,
          lastPlayed: instance.last_played ? new Date(instance.last_played) : undefined,
          totalPlayTime: instance.total_play_time || 0,
          isModded: instance.is_modded || false,
          modsCount: instance.mods_count || 0,
          isExternal: true,
          externalLauncher: instance.launcher_type as 'gdlauncher' | 'multimc' | 'prism' | 'modrinth',
          modpack: instance.modpack,
          modpackVersion: instance.modpack_version,
          icon: instance.icon,
          status,
          errorMessage,
        };
      }).filter(instance => instance.gameDir);
    } catch (error) {
      console.error('Failed to detect external instances:', error);
      return [];
    }
  }

  static async createInstance(params: {
    versionId: string;
    instanceName: string;
    gameDir: string;
    instanceId: string;
  }): Promise<void> {
    await invoke('install_minecraft_version', params);
  }

  static async launchInstance(params: {
    instanceId: string;
    instancePath: string;
    version: string;
    javaPath: string;
    memory: number;
    jvmArgs: string[];
  }): Promise<void> {
    await invoke('launch_instance', params);
  }

  static async deleteInstance(instanceId: string): Promise<void> {
    await invoke('delete_instance', { instanceId });
  }

  static async openInstanceFolder(instanceId: string): Promise<void> {
    await invoke('open_instance_folder', { instanceId });
  }

  static async importOrphanedInstances(): Promise<string[]> {
    try {
      return await invoke('import_orphaned_instances') as string[];
    } catch (error) {
      console.log('Orphaned instance scan failed (normal if no orphans):', error);
      return [];
    }
  }
}

export default InstanceService;