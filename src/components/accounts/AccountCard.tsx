import React from 'react';
import { User, LogOut } from 'lucide-react';
import { motion } from 'framer-motion';
import { MinecraftAccount } from '../../types/minecraft';

interface AccountCardProps {
  account: MinecraftAccount;
  isActive: boolean;
  onSetActive: (accountId: string) => void;
  onRemove: (accountId: string) => void;
}

export const AccountCard: React.FC<AccountCardProps> = ({
  account,
  isActive,
  onSetActive,
  onRemove,
}) => {
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

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={`bg-primary-800 border rounded-lg p-4 transition-colors ${
        isActive ? 'border-secondary-500' : 'border-primary-700 hover:border-primary-600'
      }`}
    >
      <div className="flex items-center gap-4">
        <div className="w-16 h-16 bg-primary-700 rounded-lg flex items-center justify-center">
          {account.skin_url ? (
            <img 
              src={account.skin_url} 
              alt={account.username} 
              className="w-full h-full rounded-lg object-cover"
            />
          ) : (
            <User size={32} className="text-primary-400" />
          )}
        </div>
        
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-semibold text-white">{account.username}</h3>
            <span className="text-lg">{getAccountTypeIcon('microsoft')}</span>
            {isActive && (
              <span className="bg-secondary-600 text-white text-xs px-2 py-1 rounded-full">
                Active
              </span>
            )}
          </div>
          <p className="text-primary-400 text-sm">{account.uuid}</p>
          <p className="text-primary-500 text-xs">Microsoft Account</p>
          <p className="text-primary-500 text-xs">
            Token expires: {new Date(account.expires_at * 1000).toLocaleString()}
          </p>
        </div>
        
        <div className="flex gap-2">
          {!isActive && (
            <button
              onClick={() => onSetActive(account.id)}
              className="bg-secondary-600 hover:bg-secondary-700 text-white px-3 py-1 rounded text-sm transition-colors"
            >
              Set Active
            </button>
          )}
          <button
            onClick={() => onRemove(account.id)}
            className="bg-secondary-600 hover:bg-secondary-700 text-white p-2 rounded transition-colors"
          >
            <LogOut size={16} />
          </button>
        </div>
      </div>
    </motion.div>
  );
};