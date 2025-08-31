import React, { useState } from 'react';
import { Plus, User, Key } from 'lucide-react';
import { useAccounts } from '../hooks/useAccounts';
import { AccountCard } from './accounts/AccountCard';
import { AddAccountModal } from './accounts/AddAccountModal';
import { AuthTokenModal } from './accounts/AuthTokenModal';
import { AuthTokenStatus } from './accounts/AuthTokenStatus';
import Button from './ui/Button/Button';

interface AccountsViewProps {
  onSetActiveAccount?: (accountId: string) => void;
}

const AccountsView: React.FC<AccountsViewProps> = ({
  onSetActiveAccount,
}) => {
  const {
    accounts,
    activeAccountId,
    currentAuthToken,
    isAuthenticating,
    saveAuthToken,
    clearAuthToken,
    loginWithMicrosoft,
    removeAccount,
    setActiveAccount,
    addOfflineAccount
  } = useAccounts();
  
  const [showAddModal, setShowAddModal] = useState(false);
  const [showAuthModal, setShowAuthModal] = useState(false);


  const handleSetActiveAccount = (accountId: string) => {
    setActiveAccount(accountId);
    onSetActiveAccount?.(accountId);
  };

  const handleMicrosoftLogin = async () => {
    const account = await loginWithMicrosoft();
    alert(`Successfully added Microsoft account: ${account.username}`);
  };

  const handleRemoveAccount = async (accountId: string) => {
    try {
      await removeAccount(accountId);
    } catch (error) {
      alert('Failed to remove account');
    }
  };

  const handleClearAuthToken = async () => {
    try {
      await clearAuthToken();
    } catch (error) {
      alert('Failed to clear auth token');
    }
  };

  return (
    <div className="flex-1 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Accounts</h1>
          <p className="text-primary-400">Manage your Minecraft accounts</p>
        </div>
        <div className="flex gap-2">
          <Button
            onClick={() => setShowAuthModal(true)}
            variant="primary"
            icon={Key}
          >
            Auth Token
          </Button>
          <Button
            onClick={() => setShowAddModal(true)}
            variant="primary"
            icon={Plus}
          >
            Add Account
          </Button>
        </div>
      </div>

      <AuthTokenStatus
        currentAuthToken={currentAuthToken}
        onShowModal={() => setShowAuthModal(true)}
        onClearToken={handleClearAuthToken}
      />

      <div className="space-y-4">
        {accounts.length > 0 ? (
          accounts.map((account) => (
            <AccountCard
              key={account.id}
              account={account}
              isActive={activeAccountId === account.id}
              onSetActive={handleSetActiveAccount}
              onRemove={handleRemoveAccount}
            />
          ))
        ) : (
          <div className="text-center py-12">
            <User size={64} className="mx-auto text-primary-600 mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">No accounts added</h3>
            <p className="text-primary-400 mb-4">Add an account to start playing Minecraft</p>
            <Button
              onClick={() => setShowAddModal(true)}
              variant="primary"
            >
              Add Your First Account
            </Button>
          </div>
        )}
      </div>

      <AddAccountModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onMicrosoftLogin={handleMicrosoftLogin}
        onOfflineAccount={addOfflineAccount}
        isAuthenticating={isAuthenticating}
      />

      <AuthTokenModal
        isOpen={showAuthModal}
        onClose={() => setShowAuthModal(false)}
        onSave={saveAuthToken}
      />
    </div>
  );
};

export default AccountsView;