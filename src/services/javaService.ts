import { invoke } from '@tauri-apps/api/core';

export class JavaService {
  static async getBundledJavaPath(): Promise<string> {
    return await invoke('get_bundled_java_path') as string;
  }

  static async getJavaForMinecraftVersion(minecraftVersion: string): Promise<string> {
    return await invoke('get_java_for_minecraft_version', { minecraftVersion }) as string;
  }

  static async getRequiredJavaVersion(minecraftVersion: string): Promise<number> {
    return await invoke('get_required_java_version', { minecraftVersion }) as number;
  }

  static async ensureJavaAvailable(): Promise<void> {
    try {
      await this.getBundledJavaPath();
      console.log('Bundled Java is available');
    } catch (error) {
      console.log('Bundled Java not found, will download when needed');
    }
  }

  static async getJavaForInstance(version: string): Promise<{
    javaPath: string;
    requiresInstall: boolean;
    requiredVersion?: number;
  }> {
    try {
      const javaPath = await this.getJavaForMinecraftVersion(version);
      return { javaPath, requiresInstall: false };
    } catch {
      try {
        const javaPath = await this.getBundledJavaPath();
        return { javaPath, requiresInstall: false };
      } catch {
        const requiredVersion = await this.getRequiredJavaVersion(version).catch(() => 17);
        return { javaPath: '', requiresInstall: true, requiredVersion };
      }
    }
  }
}

export default JavaService;