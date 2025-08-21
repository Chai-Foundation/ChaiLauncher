import React, { useState } from 'react';
import { Save, Folder, HardDrive, Coffee, Palette, Shield } from 'lucide-react';
import { motion } from 'framer-motion';
import { LauncherSettings } from '../types/minecraft';

interface SettingsViewProps {
  settings: LauncherSettings;
  onUpdateSettings: (settings: LauncherSettings) => void;
}

type SettingsTab = 'general' | 'java' | 'appearance' | 'advanced';

const SettingsView: React.FC<SettingsViewProps> = ({ settings, onUpdateSettings }) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [localSettings, setLocalSettings] = useState<LauncherSettings>(settings);
  const [hasChanges, setHasChanges] = useState(false);

  const handleSettingChange = (key: keyof LauncherSettings, value: any) => {
    const newSettings = { ...localSettings, [key]: value };
    setLocalSettings(newSettings);
    setHasChanges(true);
  };

  const handleSave = () => {
    onUpdateSettings(localSettings);
    setHasChanges(false);
  };

  const handleReset = () => {
    setLocalSettings(settings);
    setHasChanges(false);
  };

  const tabs = [
    { id: 'general', label: 'General', icon: HardDrive },
    { id: 'java', label: 'Java', icon: Coffee },
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'advanced', label: 'Advanced', icon: Shield },
  ];

  return (
    <div className="flex-1 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Settings</h1>
          <p className="text-gray-400">Customize your launcher experience</p>
        </div>
        {hasChanges && (
          <div className="flex gap-2">
            <button
              onClick={handleReset}
              className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
            >
              Reset
            </button>
            <button
              onClick={handleSave}
              className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors"
            >
              <Save size={18} />
              Save Changes
            </button>
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className="lg:col-span-1">
          <nav className="space-y-1">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as SettingsTab)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-left ${
                    activeTab === tab.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-400 hover:bg-gray-800 hover:text-white'
                  }`}
                >
                  <Icon size={20} />
                  {tab.label}
                </button>
              );
            })}
          </nav>
        </div>

        <div className="lg:col-span-3">
          <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
            {activeTab === 'general' && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="space-y-6"
              >
                <h3 className="text-lg font-semibold text-white mb-4">General Settings</h3>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Game Directory
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={localSettings.gameDir}
                      onChange={(e) => handleSettingChange('gameDir', e.target.value)}
                      className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button className="bg-gray-600 hover:bg-gray-700 text-white px-3 py-2 rounded-lg transition-colors">
                      <Folder size={18} />
                    </button>
                  </div>
                  <p className="text-sm text-gray-400 mt-1">
                    Directory where Minecraft instances will be stored
                  </p>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <label className="text-sm font-medium text-gray-300">
                      Keep launcher open after game starts
                    </label>
                    <p className="text-sm text-gray-400">
                      Launcher will stay open while playing
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localSettings.keepLauncherOpen}
                    onChange={(e) => handleSettingChange('keepLauncherOpen', e.target.checked)}
                    className="rounded border-gray-600 bg-gray-700"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <label className="text-sm font-medium text-gray-300">
                      Show snapshot versions
                    </label>
                    <p className="text-sm text-gray-400">
                      Include development versions in version list
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localSettings.showSnapshots}
                    onChange={(e) => handleSettingChange('showSnapshots', e.target.checked)}
                    className="rounded border-gray-600 bg-gray-700"
                  />
                </div>
              </motion.div>
            )}

            {activeTab === 'java' && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="space-y-6"
              >
                <h3 className="text-lg font-semibold text-white mb-4">Java Settings</h3>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Java Executable Path
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={localSettings.javaPath}
                      onChange={(e) => handleSettingChange('javaPath', e.target.value)}
                      placeholder="Auto-detect"
                      className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button className="bg-gray-600 hover:bg-gray-700 text-white px-3 py-2 rounded-lg transition-colors">
                      <Folder size={18} />
                    </button>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      Minimum Memory (MB)
                    </label>
                    <input
                      type="number"
                      value={localSettings.minMemory}
                      onChange={(e) => handleSettingChange('minMemory', parseInt(e.target.value))}
                      min="512"
                      max="4096"
                      className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      Maximum Memory (MB)
                    </label>
                    <input
                      type="number"
                      value={localSettings.maxMemory}
                      onChange={(e) => handleSettingChange('maxMemory', parseInt(e.target.value))}
                      min="1024"
                      max="16384"
                      className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    JVM Arguments
                  </label>
                  <textarea
                    value={localSettings.jvmArgs.join(' ')}
                    onChange={(e) => handleSettingChange('jvmArgs', e.target.value.split(' ').filter(arg => arg.trim()))}
                    placeholder="-XX:+UnlockExperimentalVMOptions -XX:+UseG1GC"
                    className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 h-24 resize-none"
                  />
                  <p className="text-sm text-gray-400 mt-1">
                    Advanced JVM arguments for performance tuning
                  </p>
                </div>
              </motion.div>
            )}

            {activeTab === 'appearance' && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="space-y-6"
              >
                <h3 className="text-lg font-semibold text-white mb-4">Appearance</h3>
                
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-3">
                    Theme
                  </label>
                  <div className="grid grid-cols-2 gap-3">
                    <button
                      onClick={() => handleSettingChange('theme', 'dark')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        localSettings.theme === 'dark'
                          ? 'border-blue-600 bg-blue-600 bg-opacity-20'
                          : 'border-gray-600 hover:border-gray-500'
                      }`}
                    >
                      <div className="w-full h-16 bg-gray-900 rounded mb-2"></div>
                      <p className="text-white font-medium">Dark</p>
                    </button>
                    
                    <button
                      onClick={() => handleSettingChange('theme', 'light')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        localSettings.theme === 'light'
                          ? 'border-blue-600 bg-blue-600 bg-opacity-20'
                          : 'border-gray-600 hover:border-gray-500'
                      }`}
                    >
                      <div className="w-full h-16 bg-gray-200 rounded mb-2"></div>
                      <p className="text-white font-medium">Light</p>
                    </button>
                  </div>
                </div>
              </motion.div>
            )}

            {activeTab === 'advanced' && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="space-y-6"
              >
                <h3 className="text-lg font-semibold text-white mb-4">Advanced Settings</h3>
                
                <div className="bg-yellow-900 bg-opacity-50 border border-yellow-700 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Shield className="text-yellow-500" size={20} />
                    <h4 className="font-semibold text-yellow-300">Warning</h4>
                  </div>
                  <p className="text-yellow-200 text-sm">
                    These settings are for advanced users only. Changing these values may cause instability.
                  </p>
                </div>

                <div className="space-y-4">
                  <button className="w-full bg-red-600 hover:bg-red-700 text-white py-2 px-4 rounded-lg transition-colors">
                    Clear All Instance Data
                  </button>
                  
                  <button className="w-full bg-orange-600 hover:bg-orange-700 text-white py-2 px-4 rounded-lg transition-colors">
                    Reset Launcher Settings
                  </button>
                  
                  <button className="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded-lg transition-colors">
                    Export Settings
                  </button>
                  
                  <button className="w-full bg-purple-600 hover:bg-purple-700 text-white py-2 px-4 rounded-lg transition-colors">
                    Import Settings
                  </button>
                </div>
              </motion.div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsView;