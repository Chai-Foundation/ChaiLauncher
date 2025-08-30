import { useState, useEffect } from 'react';
import { MinecraftAccount } from '../types/minecraft';

export const useAccounts = () => {
  const [accounts, setAccounts] = useState<MinecraftAccount[]>([]);
  const [activeAccountId, setActiveAccountId] = useState<string | null>(null);
  const [currentAuthToken, setCurrentAuthToken] = useState<string | null>(null);
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  useEffect(() => {
    loadAuthToken();
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const storedAccounts = await invoke('get_stored_accounts') as MinecraftAccount[];
      setAccounts(storedAccounts);
      
      if (storedAccounts.length > 0 && !activeAccountId) {
        setActiveAccountId(storedAccounts[0].id);
      }
    } catch (error) {
      console.error('Failed to load accounts:', error);
    }
  };

  const loadAuthToken = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const token = await invoke('get_auth_token') as string | null;
      setCurrentAuthToken(token);
    } catch (error) {
      console.error('Failed to load auth token:', error);
    }
  };

  const saveAuthToken = async (token: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_auth_token', { token });
      setCurrentAuthToken(token);
    } catch (error) {
      console.error('Failed to save auth token:', error);
      throw error;
    }
  };

  const clearAuthToken = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clear_auth_token');
      setCurrentAuthToken(null);
    } catch (error) {
      console.error('Failed to clear auth token:', error);
      throw error;
    }
  };

  const loginWithMicrosoft = async () => {
    try {
      setIsAuthenticating(true);
      const { invoke } = await import('@tauri-apps/api/core');
      
      const account = await invoke('start_oauth_with_server') as MinecraftAccount;
      await loadAccounts();
      
      return account;
    } catch (error) {
      console.error('Microsoft login failed:', error);
      throw error;
    } finally {
      setIsAuthenticating(false);
    }
  };

  const removeAccount = async (accountId: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_minecraft_account', { accountId });
      await loadAccounts();
      
      if (activeAccountId === accountId) {
        setActiveAccountId(accounts.length > 1 ? accounts.find(a => a.id !== accountId)?.id || null : null);
      }
    } catch (error) {
      console.error('Failed to remove account:', error);
      throw error;
    }
  };

  const setActiveAccount = (accountId: string) => {
    setActiveAccountId(accountId);
  };

  const addOfflineAccount = (username: string) => {
    const newAccount: MinecraftAccount = {
      id: `offline_${Date.now()}`,
      username: username.trim(),
      uuid: `offline-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      access_token: 'offline',
      refresh_token: 'offline',
      expires_at: Date.now() + (365 * 24 * 60 * 60 * 1000),
      type: 'offline',
      isActive: false,
      lastUsed: new Date()
    };
    
    setAccounts(prev => [...prev, newAccount]);
    
    if (accounts.length === 0) {
      setActiveAccountId(newAccount.id);
    }
  };

  return {
    accounts,
    activeAccountId,
    currentAuthToken,
    isAuthenticating,
    loadAccounts,
    saveAuthToken,
    clearAuthToken,
    loginWithMicrosoft,
    removeAccount,
    setActiveAccount,
    addOfflineAccount
  };
};