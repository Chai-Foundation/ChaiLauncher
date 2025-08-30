import React, { useState } from 'react';
import { motion } from 'framer-motion';

interface AuthTokenModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (token: string) => Promise<void>;
}

export const AuthTokenModal: React.FC<AuthTokenModalProps> = ({
  isOpen,
  onClose,
  onSave,
}) => {
  const [authToken, setAuthToken] = useState('');

  if (!isOpen) return null;

  const handleSubmit = async () => {
    try {
      await onSave(authToken);
      setAuthToken('');
      onClose();
    } catch (error) {
      alert('Failed to save auth token');
    }
  };

  const handleClose = () => {
    setAuthToken('');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="bg-primary-800 rounded-lg border border-primary-700 p-6 w-full max-w-md"
      >
        <h3 className="text-lg font-semibold text-white mb-4">Set Authentication Token</h3>
        
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Authentication Token
            </label>
            <input
              type="password"
              value={authToken}
              onChange={(e) => setAuthToken(e.target.value)}
              placeholder="Enter your authentication token"
              className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
            />
            <p className="text-xs text-primary-500 mt-1">
              This token will be passed to the game when launching instances.
            </p>
          </div>
          
          <div className="flex gap-2">
            <button
              onClick={handleClose}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white py-2 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={!authToken.trim()}
              className="flex-1 bg-secondary-600 hover:bg-secondary-700 text-white py-2 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Save Token
            </button>
          </div>
        </div>
      </motion.div>
    </div>
  );
};