import React from 'react';

interface AuthTokenStatusProps {
  currentAuthToken: string | null;
  onShowModal: () => void;
  onClearToken: () => void;
}

export const AuthTokenStatus: React.FC<AuthTokenStatusProps> = ({
  currentAuthToken,
  onShowModal,
  onClearToken,
}) => {
  return (
    <div className="mb-6 bg-primary-800 border border-primary-700 rounded-lg p-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-semibold text-white mb-1">Authentication Token</h3>
          <p className="text-primary-400 text-sm">
            {currentAuthToken 
              ? `Token configured (${currentAuthToken.substring(0, 8)}...)`
              : 'No authentication token configured'
            }
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={onShowModal}
            className="bg-secondary-600 hover:bg-secondary-700 text-white px-3 py-1 rounded text-sm transition-colors"
          >
            {currentAuthToken ? 'Update' : 'Set Token'}
          </button>
          {currentAuthToken && (
            <button
              onClick={onClearToken}
              className="bg-secondary-600 hover:bg-secondary-700 text-white px-3 py-1 rounded text-sm transition-colors"
            >
              Clear
            </button>
          )}
        </div>
      </div>
    </div>
  );
};