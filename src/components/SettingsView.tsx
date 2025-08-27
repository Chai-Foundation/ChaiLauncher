import React, { useState } from 'react';
import { Save, Folder, HardDrive, Coffee, Palette, Shield } from 'lucide-react';
import { motion } from 'framer-motion';
import { LauncherSettings } from '../types/minecraft';
import { getCurrentPrimaryColor, getCurrentSecondaryColor } from '../utils/colors';

interface SettingsViewProps {
  settings: LauncherSettings;
  onUpdateSettings: (settings: LauncherSettings) => void;
  onOpenFolder?: (path: string) => void;
}

type SettingsTab = 'general' | 'java' | 'appearance' | 'advanced';

const SettingsView: React.FC<SettingsViewProps> = ({ settings, onUpdateSettings, onOpenFolder }) => {
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
          <p className="text-primary-400">Customize your launcher experience</p>
        </div>
        {hasChanges && (
          <div className="flex gap-2">
            <button
              onClick={handleReset}
              className="px-4 py-2 text-primary-400 hover:text-white transition-colors"
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
                      ? 'bg-secondary-600 text-white'
                      : 'text-primary-400 hover:bg-primary-800 hover:text-white'
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
          <div className="bg-primary-800 border border-primary-700 rounded-lg p-6">
            {activeTab === 'general' && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="space-y-6"
              >
                <h3 className="text-lg font-semibold text-white mb-4">General Settings</h3>
                
                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Instances Directory
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={localSettings.instances_dir}
                      onChange={(e) => handleSettingChange('instances_dir', e.target.value)}
                      className="flex-1 px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                    <button 
                      onClick={() => onOpenFolder && onOpenFolder(localSettings.instances_dir)}
                      className="bg-primary-600 hover:bg-primary-700 text-white px-3 py-2 rounded-lg transition-colors"
                    >
                      <Folder size={18} />
                    </button>
                  </div>
                  <p className="text-sm text-primary-400 mt-1">
                    Directory where Minecraft instances will be stored
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Downloads Directory
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={localSettings.downloads_dir}
                      onChange={(e) => handleSettingChange('downloads_dir', e.target.value)}
                      className="flex-1 px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                    <button 
                      onClick={() => onOpenFolder && onOpenFolder(localSettings.downloads_dir)}
                      className="bg-primary-600 hover:bg-primary-700 text-white px-3 py-2 rounded-lg transition-colors"
                    >
                      <Folder size={18} />
                    </button>
                  </div>
                  <p className="text-sm text-primary-400 mt-1">
                    Directory where downloads will be stored
                  </p>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <label className="text-sm font-medium text-primary-300">
                      Keep launcher open after game starts
                    </label>
                    <p className="text-sm text-primary-400">
                      Launcher will stay open while playing
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localSettings.keepLauncherOpen}
                    onChange={(e) => handleSettingChange('keepLauncherOpen', e.target.checked)}
                    className="rounded border-primary-600 bg-primary-700"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <label className="text-sm font-medium text-primary-300">
                      Show snapshot versions
                    </label>
                    <p className="text-sm text-primary-400">
                      Include development versions in version list
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localSettings.showSnapshots}
                    onChange={(e) => handleSettingChange('showSnapshots', e.target.checked)}
                    className="rounded border-primary-600 bg-primary-700"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <label className="text-sm font-medium text-primary-300">
                      Auto-update launcher
                    </label>
                    <p className="text-sm text-primary-400">
                      Automatically check for and install updates
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localSettings.auto_update}
                    onChange={(e) => handleSettingChange('auto_update', e.target.checked)}
                    className="rounded border-primary-600 bg-primary-700"
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
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Java Executable Path
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={localSettings.default_java_path || ''}
                      onChange={(e) => handleSettingChange('default_java_path', e.target.value || undefined)}
                      placeholder="Auto-detect"
                      className="flex-1 px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                    <button className="bg-primary-600 hover:bg-primary-700 text-white px-3 py-2 rounded-lg transition-colors">
                      <Folder size={18} />
                    </button>
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Default Memory (MB)
                  </label>
                  <input
                    type="number"
                    value={localSettings.default_memory}
                    onChange={(e) => handleSettingChange('default_memory', parseInt(e.target.value))}
                    min="1024"
                    max="16384"
                    className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Default JVM Arguments
                  </label>
                  <textarea
                    value={localSettings.default_jvm_args.join(' ')}
                    onChange={(e) => handleSettingChange('default_jvm_args', e.target.value.split(' ').filter(arg => arg.trim()))}
                    placeholder="-XX:+UnlockExperimentalVMOptions -XX:+UseG1GC"
                    className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-secondary-500 h-24 resize-none"
                  />
                  <p className="text-sm text-primary-400 mt-1">
                    Default JVM arguments for new instances
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
                
                {/* Background Image Selection */}
                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-3">
                    Background Image
                  </label>
                  <div className="flex gap-3">
                    <input
                      type="text"
                      value={localSettings.background_image || ''}
                      onChange={(e) => handleSettingChange('background_image', e.target.value || undefined)}
                      placeholder="Path to background image (optional)"
                      className="flex-1 px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                    <button
                      onClick={() => onOpenFolder && onOpenFolder('images')}
                      className="px-3 py-2 bg-primary-600 hover:bg-primary-500 text-white rounded-lg transition-colors"
                      title="Browse for image"
                    >
                      <Folder size={18} />
                    </button>
                  </div>
                  <p className="text-sm text-primary-400 mt-1">
                    Leave empty to use default background
                  </p>
                </div>

                {/* Color Scheme Preview */}
                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-3">
                    Current Color Scheme
                  </label>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="p-4 rounded-lg border border-primary-600 bg-primary-800/50">
                      <div 
                        className="w-full h-16 rounded mb-2"
                        style={{ backgroundColor: localSettings.primary_base_color || '#78716c' }}
                      ></div>
                      <p className="text-white font-medium">Primary</p>
                      <p className="text-primary-400 text-xs">Backgrounds & structure</p>
                    </div>
                    
                    <div className="p-4 rounded-lg border border-primary-600 bg-primary-800/50">
                      <div 
                        className="w-full h-16 rounded mb-2"
                        style={{ backgroundColor: localSettings.secondary_base_color || '#d97706' }}
                      ></div>
                      <p className="text-white font-medium">Secondary</p>
                      <p className="text-primary-400 text-xs">Buttons & highlights</p>
                    </div>
                  </div>
                </div>

                {/* Base Color Customization */}
                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-3">
                    Base Color Customization
                  </label>
                  <div className="space-y-3">
                    <div className="flex items-center gap-3">
                      <label className="w-20 text-sm text-primary-400">Primary:</label>
                      <input
                        type="color"
                        value={localSettings.primary_base_color || '#78716c'}
                        onChange={(e) => handleSettingChange('primary_base_color', e.target.value)}
                        className="w-12 h-8 rounded border border-primary-600 bg-primary-700"
                      />
                      <input
                        type="text"
                        value={localSettings.primary_base_color || '#78716c'}
                        onChange={(e) => handleSettingChange('primary_base_color', e.target.value)}
                        className="flex-1 px-3 py-1 bg-primary-700 border border-primary-600 rounded text-white text-sm focus:outline-none focus:ring-1 focus:ring-secondary-500"
                        placeholder="#78716c"
                      />
                    </div>
                    <div className="flex items-center gap-3">
                      <label className="w-20 text-sm text-primary-400">Secondary:</label>
                      <input
                        type="color"
                        value={localSettings.secondary_base_color || '#d97706'}
                        onChange={(e) => handleSettingChange('secondary_base_color', e.target.value)}
                        className="w-12 h-8 rounded border border-primary-600 bg-primary-700"
                      />
                      <input
                        type="text"
                        value={localSettings.secondary_base_color || '#d97706'}
                        onChange={(e) => handleSettingChange('secondary_base_color', e.target.value)}
                        className="flex-1 px-3 py-1 bg-primary-700 border border-primary-600 rounded text-white text-sm focus:outline-none focus:ring-1 focus:ring-secondary-500"
                        placeholder="#d97706"
                      />
                    </div>
                  </div>
                  <p className="text-sm text-primary-400 mt-2">
                    Primary colors control backgrounds and structural elements. Secondary colors control buttons, highlights, and interactive elements.
                  </p>
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
                  
                  <button className="w-full bg-secondary-600 hover:bg-secondary-700 text-white py-2 px-4 rounded-lg transition-colors">
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