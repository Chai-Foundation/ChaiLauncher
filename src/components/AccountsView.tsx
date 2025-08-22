import React, { useState, useEffect } from 'react';
import { Plus, User, LogOut, Shield, EyeOff, Key } from 'lucide-react';
import { motion } from 'framer-motion';

interface MinecraftAccount {
  id: string;
  username: string;
  uuid: string;
  access_token: string;
  refresh_token: string;
  expires_at: number;
  skin_url?: string;
  cape_url?: string;
  type?: 'microsoft' | 'offline';
  isActive?: boolean;
  lastUsed?: Date;
}

interface OAuthSession {
  csrf_token: string;
  pkce_verifier: string;
  auth_url: string;
}

interface AccountsViewProps {
  onSetActiveAccount?: (accountId: string) => void;
}

const AccountsView: React.FC<AccountsViewProps> = ({
  onSetActiveAccount,
}) => {
  const [accounts, setAccounts] = useState<MinecraftAccount[]>([]);
  const [activeAccountId, setActiveAccountId] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showOfflineForm, setShowOfflineForm] = useState(false);
  const [offlineUsername, setOfflineUsername] = useState('');
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [authToken, setAuthToken] = useState('');
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
      
      // Set first account as active if none is set
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

  const handleSaveAuthToken = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_auth_token', { token: authToken });
      setCurrentAuthToken(authToken);
      setShowAuthModal(false);
      setAuthToken('');
    } catch (error) {
      console.error('Failed to save auth token:', error);
      alert('Failed to save auth token');
    }
  };

  const handleClearAuthToken = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clear_auth_token');
      setCurrentAuthToken(null);
    } catch (error) {
      console.error('Failed to clear auth token:', error);
      alert('Failed to clear auth token');
    }
  };

  const handleMicrosoftLogin = async () => {
    try {
      setIsAuthenticating(true);
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Start OAuth flow with local server
      const account = await invoke('start_oauth_with_server') as MinecraftAccount;
      
      // Refresh accounts list
      await loadAccounts();
      setShowAddModal(false);
      
      alert(`Successfully added Microsoft account: ${account.username}`);
      
    } catch (error) {
      console.error('Microsoft login failed:', error);
      alert(`Microsoft login failed: ${error}`);
    } finally {
      setIsAuthenticating(false);
    }
  };

  const handleRemoveAccount = async (accountId: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_minecraft_account', { accountId });
      await loadAccounts();
      
      // Clear active account if it was removed
      if (activeAccountId === accountId) {
        setActiveAccountId(accounts.length > 1 ? accounts.find(a => a.id !== accountId)?.id || null : null);
      }
    } catch (error) {
      console.error('Failed to remove account:', error);
      alert('Failed to remove account');
    }
  };

  const handleSetActiveAccount = (accountId: string) => {
    setActiveAccountId(accountId);
    onSetActiveAccount?.(accountId);
  };

  const handleAddOfflineAccount = () => {
    if (offlineUsername.trim()) {
      // TODO: Implement offline account creation
      console.log('Adding offline account:', offlineUsername);
      setOfflineUsername('');
      setShowOfflineForm(false);
    }
  };

  const getAccountTypeIcon = (type: string) => {
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
  };

  const getAccountTypeLabel = (type: string) => {
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
  };

  return (
    <div className="flex-1 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Accounts</h1>
          <p className="text-stone-400">Manage your Minecraft accounts</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowAuthModal(true)}
            className="bg-amber-600 hover:bg-amber-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors"
          >
            <Key size={18} />
            Auth Token
          </button>
          <button
            onClick={() => setShowAddModal(true)}
            className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors"
          >
            <Plus size={18} />
            Add Account
          </button>
        </div>
      </div>

      {/* Auth Token Status */}
      <div className="mb-6 bg-stone-800 border border-stone-700 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-semibold text-white mb-1">Authentication Token</h3>
            <p className="text-stone-400 text-sm">
              {currentAuthToken 
                ? `Token configured (${currentAuthToken.substring(0, 8)}...)`
                : 'No authentication token configured'
              }
            </p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setShowAuthModal(true)}
              className="bg-amber-600 hover:bg-amber-700 text-white px-3 py-1 rounded text-sm transition-colors"
            >
              {currentAuthToken ? 'Update' : 'Set Token'}
            </button>
            {currentAuthToken && (
              <button
                onClick={handleClearAuthToken}
                className="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded text-sm transition-colors"
              >
                Clear
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="space-y-4">
        {accounts.length > 0 ? (
          accounts.map((account) => (
            <motion.div
              key={account.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className={`bg-stone-800 border rounded-lg p-4 transition-colors ${
                activeAccountId === account.id ? 'border-green-500' : 'border-stone-700 hover:border-stone-600'
              }`}
            >
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 bg-stone-700 rounded-lg flex items-center justify-center">
                  {account.skin_url ? (
                    <img 
                      src={account.skin_url} 
                      alt={account.username} 
                      className="w-full h-full rounded-lg object-cover"
                    />
                  ) : (
                    <User size={32} className="text-stone-400" />
                  )}
                </div>
                
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="font-semibold text-white">{account.username}</h3>
                    <span className="text-lg">{getAccountTypeIcon('microsoft')}</span>
                    {activeAccountId === account.id && (
                      <span className="bg-green-600 text-white text-xs px-2 py-1 rounded-full">
                        Active
                      </span>
                    )}
                  </div>
                  <p className="text-stone-400 text-sm">{account.uuid}</p>
                  <p className="text-stone-500 text-xs">Microsoft Account</p>
                  <p className="text-stone-500 text-xs">
                    Token expires: {new Date(account.expires_at * 1000).toLocaleString()}
                  </p>
                </div>
                
                <div className="flex gap-2">
                  {activeAccountId !== account.id && (
                    <button
                      onClick={() => handleSetActiveAccount(account.id)}
                      className="bg-amber-600 hover:bg-amber-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Set Active
                    </button>
                  )}
                  <button
                    onClick={() => handleRemoveAccount(account.id)}
                    className="bg-red-600 hover:bg-red-700 text-white p-2 rounded transition-colors"
                  >
                    <LogOut size={16} />
                  </button>
                </div>
              </div>
            </motion.div>
          ))
        ) : (
          <div className="text-center py-12">
            <User size={64} className="mx-auto text-stone-600 mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">No accounts added</h3>
            <p className="text-stone-400 mb-4">Add an account to start playing Minecraft</p>
            <button
              onClick={() => setShowAddModal(true)}
              className="bg-amber-600 hover:bg-amber-700 text-white px-6 py-2 rounded-lg transition-colors"
            >
              Add Your First Account
            </button>
          </div>
        )}
      </div>

      {showAddModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="bg-stone-800 rounded-lg border border-stone-700 p-6 w-full max-w-md"
          >
            <h3 className="text-lg font-semibold text-white mb-4">Add Account</h3>
            
            {!showOfflineForm ? (
              <div className="space-y-3">
                <button
                  onClick={handleMicrosoftLogin}
                  disabled={isAuthenticating}
                  className="w-full bg-green-600 hover:bg-green-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Shield size={20} />
                  <div className="text-left">
                    <div className="font-semibold">
                      {isAuthenticating ? 'Authenticating...' : 'Microsoft Account'}
                    </div>
                    <div className="text-sm opacity-90">Recommended for most users</div>
                  </div>
                </button>
                
                <button
                  onClick={() => setShowOfflineForm(true)}
                  className="w-full bg-stone-600 hover:bg-stone-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors"
                >
                  <EyeOff size={20} />
                  <div className="text-left">
                    <div className="font-semibold">Offline Account</div>
                    <div className="text-sm opacity-90">Play without authentication (Coming soon)</div>
                  </div>
                </button>
              </div>
            ) : (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-stone-300 mb-2">
                    Username
                  </label>
                  <input
                    type="text"
                    value={offlineUsername}
                    onChange={(e) => setOfflineUsername(e.target.value)}
                    placeholder="Enter username"
                    className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
                  />
                </div>
                
                <div className="flex gap-2">
                  <button
                    onClick={() => setShowOfflineForm(false)}
                    className="flex-1 bg-stone-600 hover:bg-stone-700 text-white py-2 rounded-lg transition-colors"
                  >
                    Back
                  </button>
                  <button
                    onClick={handleAddOfflineAccount}
                    disabled={!offlineUsername.trim()}
                    className="flex-1 bg-green-600 hover:bg-green-700 text-white py-2 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Add Account
                  </button>
                </div>
              </div>
            )}
            
            <div className="flex justify-end mt-6">
              <button
                onClick={() => {
                  setShowAddModal(false);
                  setShowOfflineForm(false);
                  setOfflineUsername('');
                }}
                className="text-stone-400 hover:text-white transition-colors"
              >
                Cancel
              </button>
            </div>
          </motion.div>
        </div>
      )}

      {/* Auth Token Modal */}
      {showAuthModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="bg-stone-800 rounded-lg border border-stone-700 p-6 w-full max-w-md"
          >
            <h3 className="text-lg font-semibold text-white mb-4">Set Authentication Token</h3>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-stone-300 mb-2">
                  Authentication Token
                </label>
                <input
                  type="password"
                  value={authToken}
                  onChange={(e) => setAuthToken(e.target.value)}
                  placeholder="Enter your authentication token"
                  className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
                />
                <p className="text-xs text-stone-500 mt-1">
                  This token will be passed to the game when launching instances.
                </p>
              </div>
              
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    setShowAuthModal(false);
                    setAuthToken('');
                  }}
                  className="flex-1 bg-stone-600 hover:bg-stone-700 text-white py-2 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveAuthToken}
                  disabled={!authToken.trim()}
                  className="flex-1 bg-amber-600 hover:bg-amber-700 text-white py-2 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Save Token
                </button>
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </div>
  );
};

export default AccountsView;