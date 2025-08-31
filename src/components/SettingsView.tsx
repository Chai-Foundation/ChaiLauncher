import React, { useState } from 'react';
import { Save, Folder, HardDrive, Coffee, Palette, Shield, RotateCcw } from 'lucide-react';
import { motion } from 'framer-motion';
import { LauncherSettings } from '../types/minecraft';
import { getCurrentPrimaryColor, getCurrentSecondaryColor, DEFAULT_PRIMARY_COLOR, DEFAULT_SECONDARY_COLOR } from '../utils/colors';
import Button from './ui/Button/Button';

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
  const [backgroundImageInput, setBackgroundImageInput] = useState('');

  // Helper function to truncate display value for performance
  const getDisplayValue = (value: string | undefined): string => {
    if (!value) return '';
    if (value.startsWith('data:')) {
      // For data URLs, show a friendly description instead of the full URL
      return `[Data URL - ${Math.round(value.length / 1024)}KB]`;
    }
    if (value.length > 80) {
      // For long paths, show beginning and end
      return `${value.substring(0, 40)}...${value.substring(value.length - 35)}`;
    }
    return value;
  };

  // Update input display when settings change
  React.useEffect(() => {
    setBackgroundImageInput(getDisplayValue(localSettings.background_image));
  }, [localSettings.background_image]);

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

  const handleBackgroundImageInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setBackgroundImageInput(value);
    
    // Only update the actual setting if it's not a truncated display value
    if (!value.includes('[Data URL -') && !value.includes('...')) {
      handleSettingChange('background_image', value || undefined);
    }
  };

  const handleBackgroundImageSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Check file size (limit to 10MB for performance)
    if (file.size > 10 * 1024 * 1024) {
      alert('Background image file is too large. Please choose a file under 10MB.');
      e.target.value = ''; // Clear the input
      return;
    }

    try {
      // Use file path if available (Tauri/desktop), otherwise use data URL (web)
      const path = (file as any).path;
      if (path) {
        // On desktop, use the file path
        handleSettingChange('background_image', path.includes(' ') ? `"${path}"` : path);
      } else {
        // On web, use data URL but with size limit already checked
        const reader = new FileReader();
        reader.onload = (event) => {
          const dataUrl = event.target?.result as string;
          if (dataUrl) {
            handleSettingChange('background_image', dataUrl);
          }
        };
        reader.readAsDataURL(file);
      }
    } catch (error) {
      console.error('Failed to read background image:', error);
      alert('Failed to load the background image. Please try a different file.');
    }
    
    // Clear the input to allow selecting the same file again if needed
    e.target.value = '';
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
            <Button
              onClick={handleReset}
              variant="ghost"
            >
              Reset
            </Button>
            <Button
              onClick={handleSave}
              variant="primary"
              icon={Save}
            >
              Save Changes
            </Button>
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
                    <Button
                      onClick={() => onOpenFolder && onOpenFolder(localSettings.instances_dir)}
                      variant="secondary"
                      icon={Folder}
                    />
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
                    <Button
                      onClick={() => onOpenFolder && onOpenFolder(localSettings.downloads_dir)}
                      variant="secondary"
                      icon={Folder}
                    />
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
                      className="flex-1 px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                    <Button
                      variant="secondary"
                      icon={Folder}
                    />
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
                    className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500 h-24 resize-none"
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
                  <div className="flex gap-2">
                    <div className="flex-1 relative">
                      <input
                        type="text"
                        value={backgroundImageInput}
                        onChange={handleBackgroundImageInputChange}
                        placeholder="Path to background image (optional)"
                        className="w-full px-3 py-2 pr-10 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500 font-mono text-sm"
                        title={localSettings.background_image || 'Enter file path or select image'}
                      />
                      {localSettings.background_image && localSettings.background_image !== backgroundImageInput && (
                        <div className="absolute inset-y-0 right-8 flex items-center">
                          <div className="w-2 h-2 bg-secondary-500 rounded-full animate-pulse" title="File loaded" />
                        </div>
                      )}
                      {localSettings.background_image && (
                        <button
                          type="button"
                          onClick={() => {
                            handleSettingChange('background_image', undefined);
                            setBackgroundImageInput('');
                          }}
                          className="absolute inset-y-0 right-2 flex items-center text-primary-400 hover:text-red-400 transition-colors"
                          title="Clear background image"
                        >
                          ×
                        </button>
                      )}
                    </div>
                    <input
                      type="file"
                      accept="image/*"
                      onChange={handleBackgroundImageSelect}
                      className="hidden"
                      id="bg-image-picker"
                    />
                    <Button
                      type="button"
                      onClick={() => document.getElementById('bg-image-picker')?.click()}
                      variant="secondary"
                      icon={Folder}
                    />
                  </div>
                  <p className="text-sm text-primary-400 mt-1">
                    Leave empty to use default background. Large files and long paths are truncated for performance.
                    {localSettings.background_image && localSettings.background_image !== backgroundImageInput && (
                      <span className="block mt-1 text-secondary-400">
                        ✓ Background image loaded ({localSettings.background_image.startsWith('data:') 
                          ? `${Math.round(localSettings.background_image.length / 1024)}KB data URL` 
                          : 'file path'})
                      </span>
                    )}
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
                        style={{ backgroundColor: localSettings.primary_base_color || DEFAULT_PRIMARY_COLOR }}
                      ></div>
                      <p className="text-white font-medium">Primary</p>
                      <p className="text-primary-400 text-xs">Backgrounds & structure</p>
                    </div>
                    
                    <div className="p-4 rounded-lg border border-primary-600 bg-primary-800/50">
                      <div 
                        className="w-full h-16 rounded mb-2"
                        style={{ backgroundColor: localSettings.secondary_base_color || DEFAULT_SECONDARY_COLOR }}
                      ></div>
                      <p className="text-white font-medium">Secondary</p>
                      <p className="text-primary-400 text-xs">Buttons & highlights</p>
                    </div>
                  </div>
                </div>

                {/* Base Color Customization */}
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <label className="text-sm font-medium text-primary-300">
                      Base Color Customization
                    </label>
                    <Button
                      type="button"
                      onClick={() => {
                        handleSettingChange('primary_base_color', DEFAULT_PRIMARY_COLOR);
                        handleSettingChange('secondary_base_color', DEFAULT_SECONDARY_COLOR);
                      }}
                      variant="primary"
                      size="sm"
                      icon={RotateCcw}
                      title="Reset both colors to default"
                    >
                      Reset All
                    </Button>
                  </div>
                  <div className="space-y-3">
                    <div className="flex items-center gap-3">
                      <label className="w-20 text-sm text-primary-400">Primary:</label>
                      <input
                        type="color"
                        value={localSettings.primary_base_color || DEFAULT_PRIMARY_COLOR}
                        onChange={(e) => handleSettingChange('primary_base_color', e.target.value)}
                        className="w-12 h-8 rounded border border-primary-600 bg-primary-700"
                      />
                      <input
                        type="text"
                        value={localSettings.primary_base_color || DEFAULT_PRIMARY_COLOR}
                        onChange={(e) => handleSettingChange('primary_base_color', e.target.value)}
                        className="flex-1 px-3 py-1 bg-primary-700 border border-primary-600 rounded text-white text-sm focus:outline-none focus:ring-1 focus:ring-secondary-500"
                        placeholder={DEFAULT_PRIMARY_COLOR}
                      />
                      <Button
                        type="button"
                        onClick={() => handleSettingChange('primary_base_color', DEFAULT_PRIMARY_COLOR)}
                        variant="secondary"
                        size="sm"
                        icon={RotateCcw}
                        title="Reset to default"
                      />
                    </div>
                    <div className="flex items-center gap-3">
                      <label className="w-20 text-sm text-primary-400">Secondary:</label>
                      <input
                        type="color"
                        value={localSettings.secondary_base_color || DEFAULT_SECONDARY_COLOR}
                        onChange={(e) => handleSettingChange('secondary_base_color', e.target.value)}
                        className="w-12 h-8 rounded border border-primary-600 bg-primary-700"
                      />
                      <input
                        type="text"
                        value={localSettings.secondary_base_color || DEFAULT_SECONDARY_COLOR}
                        onChange={(e) => handleSettingChange('secondary_base_color', e.target.value)}
                        className="flex-1 px-3 py-1 bg-primary-700 border border-primary-600 rounded text-white text-sm focus:outline-none focus:ring-1 focus:ring-secondary-500"
                        placeholder={DEFAULT_SECONDARY_COLOR}
                      />
                      <Button
                        type="button"
                        onClick={() => handleSettingChange('secondary_base_color', DEFAULT_SECONDARY_COLOR)}
                        variant="secondary"
                        size="sm"
                        icon={RotateCcw}
                        title="Reset to default"
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
                
                <div className="bg-secondary-900/30 border border-secondary-700/50 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Shield className="text-secondary-500" size={20} />
                    <h4 className="font-semibold text-secondary-300">Warning</h4>
                  </div>
                  <p className="text-secondary-200 text-sm">
                    These settings are for advanced users only. Changing these values may cause instability.
                  </p>
                </div>

                <div className="space-y-4">
                  <Button
                    variant="danger"
                    fullWidth
                  >
                    Clear All Instance Data
                  </Button>
                  
                  <Button
                    variant="danger"
                    fullWidth
                  >
                    Reset Launcher Settings
                  </Button>
                  
                  <Button
                    variant="primary"
                    fullWidth
                  >
                    Export Settings
                  </Button>
                  
                  <Button
                    variant="primary"
                    fullWidth
                  >
                    Import Settings
                  </Button>
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