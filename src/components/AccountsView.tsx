import React, { useState } from 'react';
import { Plus, User, LogOut, Shield, EyeOff } from 'lucide-react';
import { motion } from 'framer-motion';

interface MinecraftAccount {
  id: string;
  username: string;
  email: string;
  type: 'microsoft' | 'mojang' | 'offline';
  isActive: boolean;
  skin?: string;
  lastUsed?: Date;
}

interface AccountsViewProps {
  accounts: MinecraftAccount[];
  onAddAccount: (type: 'microsoft' | 'mojang' | 'offline') => void;
  onRemoveAccount: (accountId: string) => void;
  onSetActiveAccount: (accountId: string) => void;
}

const AccountsView: React.FC<AccountsViewProps> = ({
  accounts,
  onAddAccount,
  onRemoveAccount,
  onSetActiveAccount,
}) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [showOfflineForm, setShowOfflineForm] = useState(false);
  const [offlineUsername, setOfflineUsername] = useState('');

  const handleAddOfflineAccount = () => {
    if (offlineUsername.trim()) {
      onAddAccount('offline');
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
        <button
          onClick={() => setShowAddModal(true)}
          className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors"
        >
          <Plus size={18} />
          Add Account
        </button>
      </div>

      <div className="space-y-4">
        {accounts.length > 0 ? (
          accounts.map((account) => (
            <motion.div
              key={account.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className={`bg-stone-800 border rounded-lg p-4 transition-colors ${
                account.isActive ? 'border-green-500' : 'border-stone-700 hover:border-stone-600'
              }`}
            >
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 bg-stone-700 rounded-lg flex items-center justify-center">
                  {account.skin ? (
                    <img 
                      src={account.skin} 
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
                    <span className="text-lg">{getAccountTypeIcon(account.type)}</span>
                    {account.isActive && (
                      <span className="bg-green-600 text-white text-xs px-2 py-1 rounded-full">
                        Active
                      </span>
                    )}
                  </div>
                  <p className="text-stone-400 text-sm">{account.email}</p>
                  <p className="text-stone-500 text-xs">{getAccountTypeLabel(account.type)}</p>
                  {account.lastUsed && (
                    <p className="text-stone-500 text-xs">
                      Last used: {account.lastUsed.toLocaleDateString()}
                    </p>
                  )}
                </div>
                
                <div className="flex gap-2">
                  {!account.isActive && (
                    <button
                      onClick={() => onSetActiveAccount(account.id)}
                      className="bg-amber-600 hover:bg-amber-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Set Active
                    </button>
                  )}
                  <button
                    onClick={() => onRemoveAccount(account.id)}
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
                  onClick={() => onAddAccount('microsoft')}
                  className="w-full bg-green-600 hover:bg-green-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors"
                >
                  <Shield size={20} />
                  <div className="text-left">
                    <div className="font-semibold">Microsoft Account</div>
                    <div className="text-sm opacity-90">Recommended for most users</div>
                  </div>
                </button>
                
                <button
                  onClick={() => onAddAccount('mojang')}
                  className="w-full bg-orange-600 hover:bg-orange-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors"
                >
                  <User size={20} />
                  <div className="text-left">
                    <div className="font-semibold">Mojang Account</div>
                    <div className="text-sm opacity-90">Legacy Minecraft accounts</div>
                  </div>
                </button>
                
                <button
                  onClick={() => setShowOfflineForm(true)}
                  className="w-full bg-stone-600 hover:bg-stone-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors"
                >
                  <EyeOff size={20} />
                  <div className="text-left">
                    <div className="font-semibold">Offline Account</div>
                    <div className="text-sm opacity-90">Play without authentication</div>
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
    </div>
  );
};

export default AccountsView;