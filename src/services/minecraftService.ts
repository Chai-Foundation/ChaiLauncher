import { invoke } from '@tauri-apps/api/core';
import { MinecraftVersion } from '../types/minecraft';

export class MinecraftService {
  static async loadVersions(): Promise<{ versions: MinecraftVersion[]; error?: string }> {
    try {
      const versionManifest = await invoke('get_minecraft_versions') as { versions: MinecraftVersion[] };
      return { versions: versionManifest.versions };
    } catch (error) {
      console.error('Failed to load Minecraft versions:', error);
      
      let errorMessage: string;
      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null && 'message' in error) {
        errorMessage = String((error as any).message);
      } else {
        errorMessage = String(error);
      }
      
      // Return fallback versions
      const fallbackVersions = [
        { id: '1.20.4', type: 'release' as const, releaseTime: '2023-12-07T12:00:00Z', url: '' },
        { id: '1.20.3', type: 'release' as const, releaseTime: '2023-12-05T12:00:00Z', url: '' },
      ];
      
      return {
        versions: fallbackVersions,
        error: `Failed to load Minecraft versions: ${errorMessage}`
      };
    }
  }
}

export default MinecraftService;