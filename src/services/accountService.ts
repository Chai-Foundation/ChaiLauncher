import { invoke } from '@tauri-apps/api/core';
import { MinecraftAccount } from '../types/accounts';

export class AccountService {
  static async getStoredAccounts(): Promise<MinecraftAccount[]> {
    try {
      return await invoke('get_stored_accounts') as MinecraftAccount[];
    } catch (error) {
      console.error('Failed to load accounts:', error);
      return [];
    }
  }

  static async getAuthToken(): Promise<string | null> {
    try {
      return await invoke('get_auth_token') as string | null;
    } catch (error) {
      console.error('Failed to load auth token:', error);
      return null;
    }
  }

  static async setAuthToken(token: string): Promise<void> {
    await invoke('set_auth_token', { token });
  }

  static async clearAuthToken(): Promise<void> {
    await invoke('clear_auth_token');
  }

  static async startOAuthWithServer(): Promise<MinecraftAccount> {
    return await invoke('start_oauth_with_server') as MinecraftAccount;
  }

  static async removeMinecraftAccount(accountId: string): Promise<void> {
    await invoke('remove_minecraft_account', { accountId });
  }

  static getAccountTypeIcon(type: string): string {
    switch (type) {
      case 'microsoft':
        return '🟢';
      case 'mojang':
        return '🔶';
      case 'offline':
        return '⚫';
      default:
        return '❓';
    }
  }

  static getAccountTypeLabel(type: string): string {
    switch (type) {
      case 'microsoft':
        return 'Microsoft Account';
      case 'mojang':
        return 'Mojang Account';
      case 'offline':
        return 'Offline Account';
      default:
        return 'Unknown';
    }
  }

  static createOfflineAccount(username: string): MinecraftAccount {
    return {
      id: `offline_${Date.now()}`,
      username: username.trim(),
      uuid: `offline-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      access_token: 'offline',
      refresh_token: 'offline',
      expires_at: Date.now() + (365 * 24 * 60 * 60 * 1000), // 1 year from now
      type: 'offline',
      isActive: false,
      lastUsed: new Date()
    };
  }
}

export default AccountService;