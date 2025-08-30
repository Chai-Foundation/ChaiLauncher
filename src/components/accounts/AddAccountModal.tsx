import React, { useState } from 'react';
import { Shield, EyeOff } from 'lucide-react';
import { motion } from 'framer-motion';

interface AddAccountModalProps {
  isOpen: boolean;
  onClose: () => void;
  onMicrosoftLogin: () => Promise<void>;
  onOfflineAccount: (username: string) => void;
  isAuthenticating: boolean;
}

export const AddAccountModal: React.FC<AddAccountModalProps> = ({
  isOpen,
  onClose,
  onMicrosoftLogin,
  onOfflineAccount,
  isAuthenticating,
}) => {
  const [showOfflineForm, setShowOfflineForm] = useState(false);
  const [offlineUsername, setOfflineUsername] = useState('');

  if (!isOpen) return null;

  const handleMicrosoftLogin = async () => {
    try {
      await onMicrosoftLogin();
      onClose();
    } catch (error) {
      alert(`Microsoft login failed: ${error}`);
    }
  };

  const handleOfflineSubmit = () => {
    if (offlineUsername.trim()) {
      onOfflineAccount(offlineUsername);
      setOfflineUsername('');
      setShowOfflineForm(false);
      onClose();
    }
  };

  const handleClose = () => {
    setShowOfflineForm(false);
    setOfflineUsername('');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="bg-primary-800 rounded-lg border border-primary-700 p-6 w-full max-w-md"
      >
        <h3 className="text-lg font-semibold text-white mb-4">Add Account</h3>
        
        {!showOfflineForm ? (
          <div className="space-y-3">
            <button
              onClick={handleMicrosoftLogin}
              disabled={isAuthenticating}
              className="w-full bg-secondary-600 hover:bg-secondary-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
              className="w-full bg-primary-600 hover:bg-primary-700 text-white p-3 rounded-lg flex items-center gap-3 transition-colors"
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
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Username
              </label>
              <input
                type="text"
                value={offlineUsername}
                onChange={(e) => setOfflineUsername(e.target.value)}
                placeholder="Enter username"
                className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
              />
            </div>
            
            <div className="flex gap-2">
              <button
                onClick={() => setShowOfflineForm(false)}
                className="flex-1 bg-primary-600 hover:bg-primary-700 text-white py-2 rounded-lg transition-colors"
              >
                Back
              </button>
              <button
                onClick={handleOfflineSubmit}
                disabled={!offlineUsername.trim()}
                className="flex-1 bg-secondary-600 hover:bg-secondary-700 text-white py-2 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Add Account
              </button>
            </div>
          </div>
        )}
        
        <div className="flex justify-end mt-6">
          <button
            onClick={handleClose}
            className="text-primary-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
        </div>
      </motion.div>
    </div>
  );
};