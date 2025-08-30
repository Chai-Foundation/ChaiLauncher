import React from 'react';
import { Settings, X } from 'lucide-react';

interface AdvancedServerSettingsProps {
  showAdvanced: boolean;
  setShowAdvanced: (show: boolean) => void;
  environmentVars: Record<string, string>;
  envVarInput: { key: string; value: string };
  setEnvVarInput: React.Dispatch<React.SetStateAction<{ key: string; value: string }>>;
  onAddEnvVar: () => void;
  onRemoveEnvVar: (key: string) => void;
}

export const AdvancedServerSettings: React.FC<AdvancedServerSettingsProps> = ({
  showAdvanced,
  setShowAdvanced,
  environmentVars,
  envVarInput,
  setEnvVarInput,
  onAddEnvVar,
  onRemoveEnvVar
}) => {
  return (
    <div>
      <button
        type="button"
        onClick={() => setShowAdvanced(!showAdvanced)}
        className="flex items-center gap-2 text-secondary-400 hover:text-secondary-300 transition-colors"
      >
        <Settings className="w-4 h-4" />
        Advanced Settings
      </button>
      
      {showAdvanced && (
        <div className="mt-4 space-y-4">
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Environment Variables
            </label>
            <div className="space-y-2">
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="Variable name"
                  value={envVarInput.key}
                  onChange={(e) => setEnvVarInput(prev => ({ ...prev, key: e.target.value }))}
                  className="flex-1 px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                />
                <input
                  type="text"
                  placeholder="Value"
                  value={envVarInput.value}
                  onChange={(e) => setEnvVarInput(prev => ({ ...prev, value: e.target.value }))}
                  className="flex-1 px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                />
                <button
                  type="button"
                  onClick={onAddEnvVar}
                  className="px-3 py-2 bg-secondary-600 hover:bg-secondary-700 text-white rounded-lg transition-colors"
                >
                  Add
                </button>
              </div>
              
              {Object.entries(environmentVars).map(([key, value]) => (
                <div key={key} className="flex items-center justify-between bg-primary-700/50 px-3 py-2 rounded-lg">
                  <span className="text-sm text-white">{key} = {value}</span>
                  <button
                    type="button"
                    onClick={() => onRemoveEnvVar(key)}
                    className="text-red-400 hover:text-red-300 transition-colors"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};