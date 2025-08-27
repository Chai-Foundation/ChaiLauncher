import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Settings, Package, Folder, Trash2, Download, Star, Search, Filter, Grid, List, RefreshCw, Loader, AlertCircle, ExternalLink, User, Calendar } from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';
import { ModInfo } from '../types/mods';
import { invoke } from '@tauri-apps/api/core';

interface InstanceSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  instance: MinecraftInstance;
  onUpdateInstance?: (instance: MinecraftInstance) => void;
}

export default function InstanceSettingsModal({ 
  isOpen, 
  onClose, 
  instance,
  onUpdateInstance 
}: InstanceSettingsModalProps) {
  const [activeTab, setActiveTab] = useState<'general' | 'mods' | 'resourcepacks'>('general');
  
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

  // Load installed mods for this instance
  React.useEffect(() => {
    if (isOpen && activeTab === 'mods') {
      loadInstalledMods();
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
        className="bg-primary-800 border border-primary-700 rounded-lg p-4 hover:border-primary-600 transition-colors"
      >
        <div className="flex items-start gap-3">
          {mod.icon_url ? (
            <img
              src={mod.icon_url}
              alt={mod.name}
              className="w-12 h-12 rounded-lg object-cover"
            />
          ) : (
            <div className="w-12 h-12 bg-primary-700 rounded-lg flex items-center justify-center">
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
                    className="px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors"
                  >
                    Uninstall
                  </button>
                ) : (
                  <button
                    onClick={() => installMod(mod)}
                    className="px-3 py-1 bg-green-600 hover:bg-green-700 text-white text-sm rounded transition-colors"
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
      <div>
        <label className="block text-sm font-medium text-primary-300 mb-2">
          Instance Name
        </label>
        <input
          type="text"
          value={instance.name}
          className="w-full px-3 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
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
          className="w-full px-3 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
          readOnly
        />
      </div>

      {instance.modpack && (
        <div>
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Modpack
          </label>
          <input
            type="text"
            value={`${instance.modpack} (${instance.modpackVersion || 'Unknown version'})`}
            className="w-full px-3 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
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
            className="flex-1 px-3 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
            readOnly
          />
          <button
            onClick={() => {
              if (instance.gameDir) {
                invoke('open_folder', { path: instance.gameDir });
              }
            }}
            className="px-3 py-2 bg-secondary-600 hover:bg-secondary-700 text-white rounded-lg transition-colors flex items-center gap-2"
          >
            <Folder size={16} />
            Open
          </button>
        </div>
      </div>
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
            className="w-full pl-10 pr-4 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
          />
        </div>
        
        <button
          type="submit"
          disabled={loading}
          className="px-4 py-2 bg-secondary-600 hover:bg-secondary-700 disabled:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2"
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
            className={`p-2 rounded transition-colors ${
              viewMode === 'grid' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-primary-300 hover:bg-primary-600'
            }`}
          >
            <Grid size={18} />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`p-2 rounded transition-colors ${
              viewMode === 'list' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-primary-300 hover:bg-primary-600'
            }`}
          >
            <List size={18} />
          </button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/50 border border-red-700 rounded-lg flex items-center gap-2 text-red-200">
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
                  className="px-6 py-2 bg-secondary-600 hover:bg-secondary-700 disabled:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2 mx-auto"
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
    <div className="text-center py-8 text-primary-400">
      <Package size={48} className="mx-auto mb-2" />
      <p>Resource Pack management coming soon</p>
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
            className="bg-primary-900 border border-primary-700 rounded-lg w-full max-w-4xl max-h-[80vh] overflow-hidden"
          >
            {/* Header */}
            <div className="flex items-center justify-between p-6 border-b border-primary-700">
              <div className="flex items-center gap-3">
                <Settings size={24} className="text-secondary-500" />
                <div>
                  <h2 className="text-xl font-bold text-white">Instance Settings</h2>
                  <p className="text-primary-400 text-sm">{instance.name}</p>
                </div>
              </div>
              <button
                onClick={onClose}
                className="p-2 text-primary-400 hover:text-white transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            {/* Tabs */}
            <div className="flex border-b border-primary-700">
              {[
                { id: 'general' as const, label: 'General', icon: Settings },
                { id: 'mods' as const, label: 'Mods', icon: Package },
                { id: 'resourcepacks' as const, label: 'Resource Packs', icon: Folder },
              ].map((tab) => {
                const Icon = tab.icon;
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`flex items-center gap-2 px-6 py-3 text-sm font-medium transition-colors ${
                      activeTab === tab.id
                        ? 'bg-secondary-600 text-white border-b-2 border-secondary-400'
                        : 'text-primary-300 hover:text-white hover:bg-primary-800'
                    }`}
                  >
                    <Icon size={16} />
                    {tab.label}
                  </button>
                );
              })}
            </div>

            {/* Content */}
            <div className="p-6 overflow-y-auto max-h-[60vh]">
              {activeTab === 'general' && renderGeneralTab()}
              {activeTab === 'mods' && renderModsTab()}
              {activeTab === 'resourcepacks' && renderResourcePacksTab()}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}