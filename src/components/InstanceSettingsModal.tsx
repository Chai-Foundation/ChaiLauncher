import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Settings, Package, Folder, Trash2, Download, Star, Search, Filter, Grid, List, RefreshCw, Loader, AlertCircle, ExternalLink, User, Calendar, Image, Monitor, Cpu, HardDrive, Camera, FolderOpen, Plus, Minus, MemoryStick } from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';
import { ModInfo } from '../types/mods';
import { invoke } from '@tauri-apps/api/core';

interface InstanceSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  instance: MinecraftInstance;
  onUpdateInstance?: (instance: MinecraftInstance) => void;
}

interface ScreenshotInfo {
  id: string;
  filename: string;
  path: string;
  timestamp: Date;
  size: number;
}

export default function InstanceSettingsModal({ 
  isOpen, 
  onClose, 
  instance,
  onUpdateInstance 
}: InstanceSettingsModalProps) {
  const [activeTab, setActiveTab] = useState<'general' | 'jvm' | 'mods' | 'resourcepacks' | 'screenshots'>('general');
  
  // Mod management state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<ModInfo[]>([]);
  const [installedMods, setInstalledMods] = useState<ModInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [hasMoreResults, setHasMoreResults] = useState(false);
  const [currentOffset, setCurrentOffset] = useState(0);
  const [isLoadingMore, setIsLoadingMore] = useState(false);

  // JVM settings state
  const [jvmSettings, setJvmSettings] = useState({
    javaPath: instance.javaPath || '',
    jvmArgs: instance.jvmArgs || [],
    minMemory: 2048, // MB
    maxMemory: 4096, // MB
    useCustomJava: !!instance.javaPath,
    useCustomArgs: (instance.jvmArgs && instance.jvmArgs.length > 0) || false
  });

  // Screenshots state
  const [screenshots, setScreenshots] = useState<ScreenshotInfo[]>([]);
  const [selectedScreenshot, setSelectedScreenshot] = useState<ScreenshotInfo | null>(null);
  const [screenshotLoading, setScreenshotLoading] = useState(false);

  // Resource packs state
  const [installedResourcePacks, setInstalledResourcePacks] = useState<string[]>([]);
  const [resourcePackLoading, setResourcePackLoading] = useState(false);

  // Load data based on active tab
  useEffect(() => {
    if (isOpen) {
      switch (activeTab) {
        case 'mods':
          loadInstalledMods();
          break;
        case 'screenshots':
          loadScreenshots();
          break;
        case 'resourcepacks':
          loadResourcePacks();
          break;
      }
    }
  }, [isOpen, activeTab, instance.id]);

  const loadInstalledMods = async () => {
    try {
      const mods = await invoke<ModInfo[]>('get_installed_mods', {
        instanceId: instance.id
      });
      setInstalledMods(mods);
    } catch (err) {
      console.error('Failed to load installed mods:', err);
      setInstalledMods([]);
    }
  };

  const loadScreenshots = async () => {
    setScreenshotLoading(true);
    try {
      const screenshots = await invoke<ScreenshotInfo[]>('get_instance_screenshots', {
        instanceId: instance.id
      });
      setScreenshots(screenshots);
    } catch (err) {
      console.error('Failed to load screenshots:', err);
      setScreenshots([]);
    } finally {
      setScreenshotLoading(false);
    }
  };

  const loadResourcePacks = async () => {
    setResourcePackLoading(true);
    try {
      const resourcePacks = await invoke<string[]>('get_installed_resource_packs', {
        instanceId: instance.id
      });
      setInstalledResourcePacks(resourcePacks);
    } catch (err) {
      console.error('Failed to load resource packs:', err);
      setInstalledResourcePacks([]);
    } finally {
      setResourcePackLoading(false);
    }
  };

  const saveJvmSettings = async () => {
    try {
      await invoke('update_instance_jvm_settings', {
        instanceId: instance.id,
        javaPath: jvmSettings.useCustomJava ? jvmSettings.javaPath : null,
        jvmArgs: jvmSettings.useCustomArgs ? jvmSettings.jvmArgs : [],
        minMemory: jvmSettings.minMemory,
        maxMemory: jvmSettings.maxMemory
      });
      
      if (onUpdateInstance) {
        onUpdateInstance({
          ...instance,
          javaPath: jvmSettings.useCustomJava ? jvmSettings.javaPath : undefined,
          jvmArgs: jvmSettings.useCustomArgs ? jvmSettings.jvmArgs : undefined
        });
      }
    } catch (err) {
      console.error('Failed to save JVM settings:', err);
      setError('Failed to save JVM settings');
    }
  };

  const deleteScreenshot = async (screenshot: ScreenshotInfo) => {
    try {
      await invoke('delete_screenshot', {
        screenshotPath: screenshot.path
      });
      setScreenshots(prev => prev.filter(s => s.id !== screenshot.id));
      if (selectedScreenshot?.id === screenshot.id) {
        setSelectedScreenshot(null);
      }
    } catch (err) {
      console.error('Failed to delete screenshot:', err);
      setError('Failed to delete screenshot');
    }
  };

  const openScreenshotFolder = async () => {
    try {
      await invoke('open_screenshots_folder', {
        instanceId: instance.id
      });
    } catch (err) {
      console.error('Failed to open screenshots folder:', err);
    }
  };

  const installResourcePack = async () => {
    try {
      const selected = await invoke<string[]>('select_resource_pack_files');
      if (selected && selected.length > 0) {
        await invoke('install_resource_packs', {
          instanceId: instance.id,
          resourcePackPaths: selected
        });
        await loadResourcePacks();
      }
    } catch (err) {
      console.error('Failed to install resource pack:', err);
      setError('Failed to install resource pack');
    }
  };

  const uninstallResourcePack = async (packName: string) => {
    try {
      await invoke('uninstall_resource_pack', {
        instanceId: instance.id,
        packName
      });
      await loadResourcePacks();
    } catch (err) {
      console.error('Failed to uninstall resource pack:', err);
      setError('Failed to uninstall resource pack');
    }
  };

  const searchMods = async (reset = true) => {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      setHasMoreResults(false);
      setCurrentOffset(0);
      return;
    }

    if (reset) {
      setLoading(true);
      setCurrentOffset(0);
      setSearchResults([]);
    } else {
      setIsLoadingMore(true);
    }
    
    setError(null);

    try {
      const offset = reset ? 0 : currentOffset;
      
      // Detect the installed mod loader for this instance
      let installedLoader = null;
      try {
        const loaderInfo = await invoke<any>('get_installed_mod_loader', {
          instanceId: instance.id
        });
        if (loaderInfo) {
          installedLoader = loaderInfo.name ? loaderInfo.name().toLowerCase() : null;
        }
      } catch (err) {
        console.log('No mod loader detected for instance:', instance.id);
      }

      const results = await invoke<ModInfo[]>('search_mods', {
        query: searchQuery,
        gameVersion: instance.version,
        modLoader: installedLoader,
        limit: 20,
        offset: offset
      });
      
      // Filter results to only show compatible mods
      const compatibleResults = results.filter(mod => {
        // Basic compatibility check - in a real implementation this would be more sophisticated
        return mod.game_versions?.includes(instance.version) ?? true;
      });
      
      if (reset) {
        setSearchResults(compatibleResults);
      } else {
        setSearchResults(prev => [...prev, ...compatibleResults]);
      }
      
      // Check if there are more results (if we got a full page, assume there might be more)
      setHasMoreResults(compatibleResults.length === 20);
      setCurrentOffset(offset + compatibleResults.length);
      
    } catch (err) {
      console.error('Search failed:', err);
      setError('Search failed. Please try again.');
      if (reset) {
        setSearchResults([]);
      }
    } finally {
      setLoading(false);
      setIsLoadingMore(false);
    }
  };

  const loadMoreResults = () => {
    if (!isLoadingMore && hasMoreResults) {
      searchMods(false);
    }
  };

  const installMod = async (mod: ModInfo) => {
    try {
      await invoke('install_mod', {
        instanceId: instance.id,
        modId: mod.id,
        versionId: null
      });
      // Reload installed mods
      await loadInstalledMods();
      // Remove from search results to avoid duplicate install attempts
      setSearchResults(prev => prev.filter(m => m.id !== mod.id));
    } catch (err) {
      console.error('Failed to install mod:', err);
      setError(`Failed to install ${mod.name}`);
    }
  };

  const uninstallMod = async (mod: ModInfo) => {
    try {
      await invoke('uninstall_mod', {
        instanceId: instance.id,
        modId: mod.id
      });
      // Reload installed mods
      await loadInstalledMods();
    } catch (err) {
      console.error('Failed to uninstall mod:', err);
      setError(`Failed to uninstall ${mod.name}`);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    searchMods(true);
  };

  const formatDownloads = (downloads: number) => {
    if (downloads >= 1000000) {
      return `${(downloads / 1000000).toFixed(1)}M`;
    } else if (downloads >= 1000) {
      return `${(downloads / 1000).toFixed(1)}K`;
    }
    return downloads.toString();
  };

  const formatDate = (dateString: string) => {
    try {
      return new Date(dateString).toLocaleDateString();
    } catch {
      return 'Unknown';
    }
  };

  const ModCard = ({ mod, installed = false }: { mod: ModInfo; installed?: boolean }) => {
    const getPlatformBadge = () => {
      const platformName = typeof mod.source === 'string' ? mod.source : 
                          'Direct' in mod.source ? 'Direct' : 'Unknown';
      
      const platformColors = {
        'CurseForge': 'bg-orange-600 text-white',
        'Modrinth': 'bg-green-600 text-white',
        'GitHub': 'bg-gray-600 text-white',
        'Direct': 'bg-blue-600 text-white',
        'Local': 'bg-purple-600 text-white',
        'Unknown': 'bg-gray-500 text-white'
      };

      return (
        <span className={`px-2 py-1 text-xs rounded-full ${platformColors[platformName] || platformColors.Unknown}`}>
          {platformName}
        </span>
      );
    };

    const getLoaderBadges = () => {
      if (!mod.loaders || mod.loaders.length === 0) return null;
      
      const loaderColors = {
        'forge': 'bg-red-600 text-white',
        'fabric': 'bg-secondary-600 text-white',
        'quilt': 'bg-purple-600 text-white',
        'neoforge': 'bg-orange-700 text-white',
        'optifine': 'bg-blue-700 text-white'
      };

      return (
        <div className="flex gap-1 flex-wrap">
          {mod.loaders.slice(0, 3).map((loader: string) => (
            <span
              key={loader}
              className={`px-2 py-1 text-xs rounded ${loaderColors[loader.toLowerCase() as keyof typeof loaderColors] || 'bg-gray-600 text-white'}`}
            >
              {loader.charAt(0).toUpperCase() + loader.slice(1)}
            </span>
          ))}
          {mod.loaders.length > 3 && (
            <span className="px-2 py-1 bg-gray-600 text-white text-xs rounded">
              +{mod.loaders.length - 3}
            </span>
          )}
        </div>
      );
    };

    const getVersionCompatibility = () => {
      if (!mod.game_versions || mod.game_versions.length === 0) return null;
      
      const isCompatible = mod.game_versions.includes(instance.version);
      const displayVersions = mod.game_versions.slice(0, 3);
      
      return (
        <div className="flex items-center gap-2 text-xs">
          <span className={`px-2 py-1 rounded-full ${isCompatible ? 'bg-green-900 text-green-200' : 'bg-yellow-900 text-yellow-200'}`}>
            {isCompatible ? '✓ Compatible' : '⚠ Check Version'}
          </span>
          <span className="text-primary-400">
            {displayVersions.join(', ')}
            {mod.game_versions.length > 3 && ` +${mod.game_versions.length - 3}`}
          </span>
        </div>
      );
    };

    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="bg-primary-800/30 border border-primary-700 rounded-lg p-4 hover:border-primary-600 transition-colors backdrop-blur-sm"
      >
        <div className="flex items-start gap-3">
          {mod.icon_url ? (
            <img
              src={mod.icon_url}
              alt={mod.name}
              className="w-12 h-12 rounded-lg object-cover"
            />
          ) : (
            <div className="w-12 h-12 bg-primary-700/50 rounded-lg flex items-center justify-center backdrop-blur-sm">
              <Package size={24} className="text-primary-400" />
            </div>
          )}
          
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-2">
              <h3 className="font-semibold text-white truncate">{mod.name}</h3>
              {mod.featured && (
                <Star size={16} className="text-yellow-500 fill-current" />
              )}
              {getPlatformBadge()}
            </div>
            
            <p className="text-primary-400 text-sm mb-3 line-clamp-2">
              {mod.description}
            </p>
            
            {/* Version Compatibility */}
            <div className="mb-3">
              {getVersionCompatibility()}
            </div>
            
            {/* Loader Requirements */}
            <div className="mb-3">
              {getLoaderBadges()}
            </div>
            
            <div className="flex items-center justify-between text-xs text-primary-500 mb-3">
              <div className="flex items-center gap-3">
                <span className="flex items-center gap-1">
                  <User size={12} />
                  {mod.author}
                </span>
                <span className="flex items-center gap-1">
                  <Download size={12} />
                  {formatDownloads(mod.downloads)}
                </span>
                <span className="flex items-center gap-1">
                  <Calendar size={12} />
                  {formatDate(mod.date_updated)}
                </span>
              </div>
            </div>
            
            <div className="flex items-center justify-between">
              <div className="flex gap-1">
                {mod.categories.slice(0, 2).map((category: string) => (
                  <span
                    key={category}
                    className="px-2 py-1 bg-primary-700 text-primary-300 text-xs rounded"
                  >
                    {category}
                  </span>
                ))}
                {mod.categories.length > 2 && (
                  <span className="px-2 py-1 bg-primary-700 text-primary-300 text-xs rounded">
                    +{mod.categories.length - 2}
                  </span>
                )}
              </div>
              
              <div className="flex gap-2">
                {mod.website_url && (
                  <button
                    onClick={() => window.open(mod.website_url, '_blank')}
                    className="p-1 text-primary-400 hover:text-white transition-colors"
                    title="Visit website"
                  >
                    <ExternalLink size={16} />
                  </button>
                )}
                {installed ? (
                  <button
                    onClick={() => uninstallMod(mod)}
                    className="px-3 py-1 bg-red-600/80 hover:bg-red-700 text-white text-sm rounded transition-colors backdrop-blur-sm"
                  >
                    Uninstall
                  </button>
                ) : (
                  <button
                    onClick={() => installMod(mod)}
                    className="px-3 py-1 bg-green-600/80 hover:bg-green-700 text-white text-sm rounded transition-colors backdrop-blur-sm"
                  >
                    Install
                  </button>
                )}
              </div>
            </div>
          </div>
        </div>
      </motion.div>
    );
  };

  const renderGeneralTab = () => (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Instance Name
          </label>
          <input
            type="text"
            value={instance.name}
            className="w-full px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
            readOnly
          />
        </div>
        
        <div>
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Minecraft Version
          </label>
          <input
            type="text"
            value={instance.version}
            className="w-full px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
            readOnly
          />
        </div>
      </div>

      {instance.modpack && (
        <div>
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Modpack
          </label>
          <input
            type="text"
            value={`${instance.modpack} (${instance.modpackVersion || 'Unknown version'})`}
            className="w-full px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
            readOnly
          />
        </div>
      )}

      <div>
        <label className="block text-sm font-medium text-primary-300 mb-2">
          Game Directory
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            value={instance.gameDir}
            className="flex-1 px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
            readOnly
          />
          <button
            onClick={() => {
              if (instance.gameDir) {
                invoke('open_folder', { path: instance.gameDir });
              }
            }}
            className="px-3 py-2 bg-secondary-600/80 hover:bg-secondary-700 text-white rounded-lg transition-colors flex items-center gap-2 backdrop-blur-sm"
          >
            <Folder size={16} />
            Open
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-primary-800/30 border border-primary-700 rounded-lg p-4 backdrop-blur-sm">
          <div className="flex items-center gap-2 mb-2">
            <HardDrive size={16} className="text-secondary-400" />
            <span className="text-sm font-medium text-primary-300">Storage</span>
          </div>
          <div className="text-lg font-semibold text-white">
            {instance.sizeMb ? `${(instance.sizeMb / 1024).toFixed(1)} GB` : 'Unknown'}
          </div>
        </div>
        
        <div className="bg-primary-800/30 border border-primary-700 rounded-lg p-4 backdrop-blur-sm">
          <div className="flex items-center gap-2 mb-2">
            <Package size={16} className="text-secondary-400" />
            <span className="text-sm font-medium text-primary-300">Mods</span>
          </div>
          <div className="text-lg font-semibold text-white">
            {instance.modsCount || 0}
          </div>
        </div>
        
        <div className="bg-primary-800/30 border border-primary-700 rounded-lg p-4 backdrop-blur-sm">
          <div className="flex items-center gap-2 mb-2">
            <Calendar size={16} className="text-secondary-400" />
            <span className="text-sm font-medium text-primary-300">Last Played</span>
          </div>
          <div className="text-sm text-white">
            {instance.lastPlayed ? new Date(instance.lastPlayed).toLocaleDateString() : 'Never'}
          </div>
        </div>
      </div>
    </div>
  );

  const renderJvmTab = () => (
    <div className="space-y-6">
      <div className="bg-primary-800/30 border border-primary-700 rounded-lg p-4 backdrop-blur-sm">
        <div className="flex items-center gap-2 mb-4">
          <Cpu size={20} className="text-secondary-400" />
          <h3 className="text-lg font-semibold text-white">Java Virtual Machine Settings</h3>
        </div>
        
        <div className="space-y-4">
          <div>
            <label className="flex items-center gap-2 mb-3">
              <input
                type="checkbox"
                checked={jvmSettings.useCustomJava}
                onChange={(e) => setJvmSettings(prev => ({ ...prev, useCustomJava: e.target.checked }))}
                className="rounded border-primary-600 bg-primary-800 text-secondary-600"
              />
              <span className="text-sm font-medium text-primary-300">Use Custom Java Path</span>
            </label>
            
            {jvmSettings.useCustomJava && (
              <div className="flex gap-2">
                <input
                  type="text"
                  value={jvmSettings.javaPath}
                  onChange={(e) => setJvmSettings(prev => ({ ...prev, javaPath: e.target.value }))}
                  placeholder="/path/to/java"
                  className="flex-1 px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
                />
                <button
                  onClick={async () => {
                    try {
                      const path = await invoke<string>('select_java_path');
                      if (path) {
                        setJvmSettings(prev => ({ ...prev, javaPath: path }));
                      }
                    } catch (err) {
                      console.error('Failed to select Java path:', err);
                    }
                  }}
                  className="px-3 py-2 bg-secondary-600/80 hover:bg-secondary-700 text-white rounded-lg transition-colors backdrop-blur-sm"
                >
                  Browse
                </button>
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                <Memory size={16} className="inline mr-1" />
                Minimum Memory (MB)
              </label>
              <input
                type="number"
                value={jvmSettings.minMemory}
                onChange={(e) => setJvmSettings(prev => ({ ...prev, minMemory: parseInt(e.target.value) || 1024 }))}
                min="512"
                max="32768"
                step="256"
                className="w-full px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                <Memory size={16} className="inline mr-1" />
                Maximum Memory (MB)
              </label>
              <input
                type="number"
                value={jvmSettings.maxMemory}
                onChange={(e) => setJvmSettings(prev => ({ ...prev, maxMemory: parseInt(e.target.value) || 4096 }))}
                min="1024"
                max="32768"
                step="256"
                className="w-full px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
              />
            </div>
          </div>

          <div>
            <label className="flex items-center gap-2 mb-3">
              <input
                type="checkbox"
                checked={jvmSettings.useCustomArgs}
                onChange={(e) => setJvmSettings(prev => ({ ...prev, useCustomArgs: e.target.checked }))}
                className="rounded border-primary-600 bg-primary-800 text-secondary-600"
              />
              <span className="text-sm font-medium text-primary-300">Use Custom JVM Arguments</span>
            </label>
            
            {jvmSettings.useCustomArgs && (
              <div className="space-y-2">
                {jvmSettings.jvmArgs.map((arg, index) => (
                  <div key={index} className="flex gap-2">
                    <input
                      type="text"
                      value={arg}
                      onChange={(e) => {
                        const newArgs = [...jvmSettings.jvmArgs];
                        newArgs[index] = e.target.value;
                        setJvmSettings(prev => ({ ...prev, jvmArgs: newArgs }));
                      }}
                      placeholder="-Xmx4G"
                      className="flex-1 px-3 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
                    />
                    <button
                      onClick={() => {
                        const newArgs = jvmSettings.jvmArgs.filter((_, i) => i !== index);
                        setJvmSettings(prev => ({ ...prev, jvmArgs: newArgs }));
                      }}
                      className="px-2 py-2 bg-red-600/80 hover:bg-red-700 text-white rounded-lg transition-colors backdrop-blur-sm"
                    >
                      <Minus size={16} />
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => {
                    setJvmSettings(prev => ({ ...prev, jvmArgs: [...prev.jvmArgs, ''] }));
                  }}
                  className="px-3 py-2 bg-green-600/80 hover:bg-green-700 text-white rounded-lg transition-colors flex items-center gap-2 backdrop-blur-sm"
                >
                  <Plus size={16} />
                  Add Argument
                </button>
              </div>
            )}
          </div>

          <div className="flex justify-end">
            <button
              onClick={saveJvmSettings}
              className="px-6 py-2 bg-secondary-600/80 hover:bg-secondary-700 text-white rounded-lg transition-colors backdrop-blur-sm"
            >
              Save JVM Settings
            </button>
          </div>
        </div>
      </div>
    </div>
  );

  const renderScreenshotsTab = () => (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Camera size={20} className="text-secondary-400" />
          <h3 className="text-lg font-semibold text-white">Screenshots ({screenshots.length})</h3>
        </div>
        <button
          onClick={openScreenshotFolder}
          className="px-3 py-2 bg-secondary-600/80 hover:bg-secondary-700 text-white rounded-lg transition-colors flex items-center gap-2 backdrop-blur-sm"
        >
          <FolderOpen size={16} />
          Open Folder
        </button>
      </div>

      {screenshotLoading ? (
        <div className="flex justify-center py-8">
          <Loader className="animate-spin text-secondary-400" size={32} />
        </div>
      ) : screenshots.length > 0 ? (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4 max-h-96 overflow-y-auto">
          {screenshots.map((screenshot) => (
            <div
              key={screenshot.id}
              className="relative group cursor-pointer bg-primary-800/30 border border-primary-700 rounded-lg overflow-hidden backdrop-blur-sm hover:border-secondary-500/50 transition-colors"
              onClick={() => setSelectedScreenshot(screenshot)}
            >
              <div className="aspect-video bg-primary-700 flex items-center justify-center">
                <Image size={24} className="text-primary-400" />
              </div>
              <div className="p-2">
                <div className="text-xs text-primary-300 truncate">{screenshot.filename}</div>
                <div className="text-xs text-primary-400">
                  {new Date(screenshot.timestamp).toLocaleDateString()}
                </div>
              </div>
              <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                <div className="flex gap-2">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      invoke('open_file', { path: screenshot.path });
                    }}
                    className="p-1 bg-secondary-600 hover:bg-secondary-700 text-white rounded transition-colors"
                    title="Open"
                  >
                    <ExternalLink size={16} />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteScreenshot(screenshot);
                    }}
                    className="p-1 bg-red-600 hover:bg-red-700 text-white rounded transition-colors"
                    title="Delete"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 text-primary-400">
          <Camera size={48} className="mx-auto mb-2" />
          <p>No screenshots found</p>
          <p className="text-sm">Screenshots will appear here automatically</p>
        </div>
      )}
    </div>
  );

  const renderModsTab = () => (
    <div className="space-y-4">
      {/* Search Bar */}
      <form onSubmit={handleSearch} className="flex gap-4">
        <div className="flex-1 relative">
          <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-primary-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={`Search mods compatible with ${instance.version}...`}
            className="w-full pl-10 pr-4 py-2 bg-primary-800/50 border border-primary-700 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500 backdrop-blur-sm"
          />
        </div>
        
        <button
          type="submit"
          disabled={loading}
          className="px-4 py-2 bg-secondary-600/80 hover:bg-secondary-700 disabled:bg-primary-600/50 text-white rounded-lg transition-colors flex items-center gap-2 backdrop-blur-sm"
        >
          {loading ? (
            <Loader size={16} className="animate-spin" />
          ) : (
            <Search size={16} />
          )}
          Search
        </button>
      </form>

      {/* View Mode Toggle */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white">
          {searchResults.length > 0 ? `Search Results (${searchResults.length})` : 
           installedMods.length > 0 ? `Installed Mods (${installedMods.length})` : 
           'Mods'}
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setViewMode('grid')}
            className={`p-2 rounded transition-colors backdrop-blur-sm ${
              viewMode === 'grid' ? 'bg-secondary-600/80 text-white' : 'bg-primary-700/50 text-primary-300 hover:bg-primary-600/50'
            }`}
          >
            <Grid size={18} />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`p-2 rounded transition-colors backdrop-blur-sm ${
              viewMode === 'list' ? 'bg-secondary-600/80 text-white' : 'bg-primary-700/50 text-primary-300 hover:bg-primary-600/50'
            }`}
          >
            <List size={18} />
          </button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/30 border border-red-700/50 rounded-lg flex items-center gap-2 text-red-200 backdrop-blur-sm">
          <AlertCircle size={18} />
          {error}
        </div>
      )}

      {/* Content */}
      <div className="max-h-96 overflow-y-auto">
        {searchResults.length > 0 ? (
          <div>
            <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1' : 'grid-cols-1'}`}>
              {searchResults.map((mod) => (
                <ModCard key={`search-${mod.id}`} mod={mod} />
              ))}
            </div>
            
            {/* Load More Button */}
            {hasMoreResults && (
              <div className="mt-4 text-center">
                <button
                  onClick={loadMoreResults}
                  disabled={isLoadingMore}
                  className="px-6 py-2 bg-secondary-600/80 hover:bg-secondary-700 disabled:bg-primary-600/50 text-white rounded-lg transition-colors flex items-center gap-2 mx-auto backdrop-blur-sm"
                >
                  {isLoadingMore ? (
                    <Loader size={16} className="animate-spin" />
                  ) : (
                    <Download size={16} />
                  )}
                  {isLoadingMore ? 'Loading...' : 'Load More'}
                </button>
              </div>
            )}
          </div>
        ) : installedMods.length > 0 ? (
          <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1' : 'grid-cols-1'}`}>
            {installedMods.map((mod) => (
              <ModCard key={`installed-${mod.id}`} mod={mod} installed={true} />
            ))}
          </div>
        ) : searchQuery.trim() && !loading ? (
          <div className="text-center py-8 text-primary-400">
            <Package size={48} className="mx-auto mb-2" />
            <p>No compatible mods found for "{searchQuery}"</p>
            <p className="text-sm">Try searching for mods that support {instance.version}</p>
          </div>
        ) : !searchQuery.trim() ? (
          <div className="text-center py-8 text-primary-400">
            <Package size={48} className="mx-auto mb-2" />
            <p>Search for mods to install</p>
            <p className="text-sm">Only mods compatible with {instance.version} will be shown</p>
          </div>
        ) : null}
      </div>
    </div>
  );

  const renderResourcePacksTab = () => (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Folder size={20} className="text-secondary-400" />
          <h3 className="text-lg font-semibold text-white">Resource Packs ({installedResourcePacks.length})</h3>
        </div>
        <button
          onClick={installResourcePack}
          className="px-3 py-2 bg-secondary-600/80 hover:bg-secondary-700 text-white rounded-lg transition-colors flex items-center gap-2 backdrop-blur-sm"
        >
          <Plus size={16} />
          Install Pack
        </button>
      </div>

      {resourcePackLoading ? (
        <div className="flex justify-center py-8">
          <Loader className="animate-spin text-secondary-400" size={32} />
        </div>
      ) : installedResourcePacks.length > 0 ? (
        <div className="space-y-2 max-h-96 overflow-y-auto">
          {installedResourcePacks.map((pack) => (
            <div
              key={pack}
              className="flex items-center justify-between p-3 bg-primary-800/30 border border-primary-700 rounded-lg backdrop-blur-sm"
            >
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-primary-700 rounded-lg flex items-center justify-center">
                  <Package size={20} className="text-primary-400" />
                </div>
                <div>
                  <div className="font-medium text-white">{pack}</div>
                  <div className="text-sm text-primary-400">Resource Pack</div>
                </div>
              </div>
              <button
                onClick={() => uninstallResourcePack(pack)}
                className="px-3 py-1 bg-red-600/80 hover:bg-red-700 text-white text-sm rounded transition-colors backdrop-blur-sm"
              >
                Remove
              </button>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 text-primary-400">
          <Folder size={48} className="mx-auto mb-2" />
          <p>No resource packs installed</p>
          <p className="text-sm">Click "Install Pack" to add resource packs</p>
        </div>
      )}
    </div>
  );

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
          onClick={onClose}
        >
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            onClick={(e) => e.stopPropagation()}
            className="bg-primary-900/80 backdrop-blur-md border border-primary-700/50 rounded-lg w-full max-w-6xl max-h-[85vh] overflow-hidden flex"
          >
            {/* Sidebar Navigation */}
            <div className="w-64 bg-primary-800/50 backdrop-blur-sm border-r border-primary-700/50 flex flex-col">
              {/* Header */}
              <div className="p-6 border-b border-primary-700/50">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-gradient-to-br from-secondary-600 to-secondary-700 rounded-lg flex items-center justify-center">
                    <Settings size={20} className="text-white" />
                  </div>
                  <div>
                    <h2 className="text-lg font-bold text-white">Settings</h2>
                    <p className="text-primary-400 text-sm truncate">{instance.name}</p>
                  </div>
                </div>
              </div>

              {/* Navigation */}
              <div className="flex-1 p-4 space-y-2">
                {[
                  { id: 'general' as const, label: 'General', icon: Settings, description: 'Basic instance info' },
                  { id: 'jvm' as const, label: 'JVM Settings', icon: Cpu, description: 'Java & memory settings' },
                  { id: 'mods' as const, label: 'Mods', icon: Package, description: 'Manage modifications' },
                  { id: 'resourcepacks' as const, label: 'Resource Packs', icon: Folder, description: 'Texture & sound packs' },
                  { id: 'screenshots' as const, label: 'Screenshots', icon: Camera, description: 'View saved images' },
                ].map((tab) => {
                  const Icon = tab.icon;
                  return (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`w-full flex items-start gap-3 p-3 rounded-lg text-left transition-all duration-200 group ${
                        activeTab === tab.id
                          ? 'bg-secondary-600/80 text-white backdrop-blur-sm shadow-md'
                          : 'text-primary-300 hover:text-white hover:bg-primary-700/50'
                      }`}
                    >
                      <Icon size={18} className={`mt-0.5 ${
                        activeTab === tab.id ? 'text-white' : 'text-primary-400 group-hover:text-secondary-400'
                      }`} />
                      <div className="flex-1">
                        <div className="font-medium text-sm">{tab.label}</div>
                        <div className={`text-xs mt-0.5 ${
                          activeTab === tab.id ? 'text-secondary-100' : 'text-primary-500'
                        }`}>
                          {tab.description}
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>

              {/* Close Button */}
              <div className="p-4 border-t border-primary-700/50">
                <button
                  onClick={onClose}
                  className="w-full flex items-center justify-center gap-2 p-2 text-primary-400 hover:text-white hover:bg-primary-700/50 rounded-lg transition-colors"
                >
                  <X size={16} />
                  Close
                </button>
              </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col">
              {/* Content Header */}
              <div className="p-6 border-b border-primary-700/50 bg-primary-800/30 backdrop-blur-sm">
                <div className="flex items-center gap-3">
                  {(() => {
                    const currentTab = [
                      { id: 'general', label: 'General', icon: Settings },
                      { id: 'jvm', label: 'JVM Settings', icon: Cpu },
                      { id: 'mods', label: 'Mods', icon: Package },
                      { id: 'resourcepacks', label: 'Resource Packs', icon: Folder },
                      { id: 'screenshots', label: 'Screenshots', icon: Camera },
                    ].find(tab => tab.id === activeTab);
                    const Icon = currentTab?.icon || Settings;
                    return (
                      <>
                        <Icon size={20} className="text-secondary-400" />
                        <h3 className="text-xl font-semibold text-white">{currentTab?.label}</h3>
                      </>
                    );
                  })()}
                </div>
              </div>

              {/* Content */}
              <div className="flex-1 p-6 overflow-y-auto">
                {activeTab === 'general' && renderGeneralTab()}
                {activeTab === 'jvm' && renderJvmTab()}
                {activeTab === 'mods' && renderModsTab()}
                {activeTab === 'resourcepacks' && renderResourcePacksTab()}
                {activeTab === 'screenshots' && renderScreenshotsTab()}
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}