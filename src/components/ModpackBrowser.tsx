import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { motion } from 'framer-motion';
import { 
  Search,
  Download,
  Star,
  Package,
  Filter,
  Grid,
  List,
  RefreshCw,
  Loader,
  Loader2,
  AlertCircle,
  ExternalLink,
  User,
  Calendar,
  Tag,
  FileDown,
  Upload,
  FolderOpen
} from 'lucide-react';
import { ModrinthPack, ModrinthVersion, ModpackInstallProgress, LauncherSettings } from '../types';
import { useInfiniteModpacks } from '../hooks/useInfiniteModpacks';
import { useInfiniteScroll } from '../hooks/useInfiniteScroll';
import Button from './ui/Button/Button';
import Modal from './ui/Modal/Modal';
import ProgressBar from './ui/ProgressBar/ProgressBar';

interface ModpackBrowserProps {
  onCreateInstance?: (data: {
    name: string;
    version: string;
    modpack?: string;
    modpackVersion?: string;
  }) => void;
  launcherSettings?: LauncherSettings | null;
}

export default function ModpackBrowser({ onCreateInstance, launcherSettings }: ModpackBrowserProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [selectedPlatform, setSelectedPlatform] = useState<'modrinth' | 'curseforge' | 'all'>('modrinth');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [installError, setInstallError] = useState<string | null>(null);
  const [installProgress, setInstallProgress] = useState<Map<string, ModpackInstallProgress>>(new Map());
  const [showVersionModal, setShowVersionModal] = useState(false);
  const [selectedPack, setSelectedPack] = useState<ModrinthPack | null>(null);
  const [availableVersions, setAvailableVersions] = useState<ModrinthVersion[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(false);
  
  const { modpacks, loading, hasMore, error, loadMore, refresh } = useInfiniteModpacks({
    searchQuery,
    platform: selectedPlatform,
    limit: 20
  });
  
  const { sentinelRef } = useInfiniteScroll({
    loading,
    hasMore,
    onLoadMore: loadMore,
  });


  // Listen for installation progress
  useEffect(() => {
    const setupProgressListener = async () => {
      try {
        const unlisten = await listen<ModpackInstallProgress>('modpack_install_progress', (event) => {
          setInstallProgress(prev => {
            const newMap = new Map(prev);
            newMap.set(event.payload.instance_dir, event.payload);
            return newMap;
          });
        });

        return unlisten;
      } catch (error) {
        console.error('Failed to setup progress listener:', error);
      }
    };

    setupProgressListener();
  }, []);



  const handleSearch = (e: React.FormEvent) => {
  e.preventDefault();
  setSearchQuery(searchInput);
  };

  const showVersionSelector = async (pack: ModrinthPack) => {
    if (!launcherSettings) {
      setInstallError('Launcher settings not available');
      return;
    }

    try {
      setSelectedPack(pack);
      setLoadingVersions(true);
      setShowVersionModal(true);
      
      // Fetch available versions
      const versions = await invoke<ModrinthVersion[]>('get_modpack_versions', {
        modpackId: pack.project_id,
        platform: selectedPlatform
      });
      
      setAvailableVersions(versions);
    } catch (error) {
      console.error('Failed to fetch modpack versions:', error);
      setInstallError(`Failed to fetch versions for ${pack.name}: ${error}`);
      setShowVersionModal(false);
    } finally {
      setLoadingVersions(false);
    }
  };

  const installModpack = async (pack: ModrinthPack, selectedVersion: ModrinthVersion) => {
    if (!launcherSettings) {
      setInstallError('Launcher settings not available');
      return;
    }

    try {
      // Start installation - backend will register the instance  
      const instanceName = `${pack.name}_${selectedVersion.version_number}`;
      await invoke('install_modpack', {
        request: {
          instance_name: instanceName,
          modpack_id: pack.project_id,
          version_id: selectedVersion.id,
          platform: selectedPlatform,
          game_directory: launcherSettings.instances_dir,
          install_overrides: false
        }
      });
      if (onCreateInstance) {
        // Call with dummy data just to trigger refresh - backend handles actual instance creation
        onCreateInstance({
          name: instanceName,
          version: selectedVersion.game_versions[0] || '1.20.1',
          modpack: pack.name,
          modpackVersion: selectedVersion.version_number
        });
      }

      setShowVersionModal(false);
    } catch (error) {
      console.error('Failed to install modpack:', error);
      setInstallError(`Failed to install ${pack.name}: ${error}`);
    }
  };

  const openModpackFile = async () => {
    try {
      // This would open a file dialog to select modpack files
      // For now, show a message about importing
      setInstallError('File import functionality coming soon!');
    } catch (error) {
      console.error('Failed to open file dialog:', error);
    }
  };

  const renderModpackCard = (pack: ModrinthPack, index: number) => {
    // Check for progress with any version suffix
    let progress = null;
    for (const [key, value] of installProgress.entries()) {
      if (typeof key === 'string' && key.includes(pack.name)) {
        progress = value;
        break;
      }
    }
    
    return (
      <motion.div
        key={pack.project_id}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        whileHover={{ scale: 1.02, y: -4 }}
        transition={{ 
          delay: (index % 20) * 0.05,
          hover: { duration: 0.2, ease: "easeOut" }
        }}
        className={`bg-primary-800/90 backdrop-blur-sm border border-secondary-600/30 rounded-lg overflow-hidden hover:border-secondary-500/50 hover:shadow-xl hover:shadow-secondary-500/10 transition-all duration-300 ${
          viewMode === 'list' ? 'flex' : ''
        }`}
      >
        {pack.icon_url && (
          <div className={`${viewMode === 'list' ? 'w-20 h-20 flex-shrink-0' : 'w-full h-32'} bg-primary-700 flex items-center justify-center overflow-hidden`}>
            <img 
              src={pack.icon_url} 
              alt={pack.name}
              className="w-full h-full object-cover"
              onError={(e) => {
                const target = e.target as HTMLImageElement;
                target.style.display = 'none';
                target.parentElement!.innerHTML = '<Package class="w-8 h-8 text-secondary-500" />';
              }}
            />
          </div>
        )}
        
        <div className="p-4 flex-1">
          <div className="flex items-start justify-between mb-2">
            <h3 className="font-semibold text-white truncate">{pack.name}</h3>
            <div className="flex items-center gap-1 text-sm text-secondary-400 ml-2">
              <Download className="w-4 h-4" />
              {pack.downloads.toLocaleString()}
            </div>
          </div>
          
          <p className="text-sm text-gray-300 mb-3 line-clamp-2">{pack.description}</p>
          
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-xs text-gray-400">
              <User className="w-3 h-3" />
              <span>{pack.author}</span>
              {pack.game_versions.length > 0 && (
                <>
                  <span>•</span>
                  <span>{pack.game_versions[0]}</span>
                </>
              )}
            </div>
            
            {progress ? (
              <motion.div 
                className="w-32"
                initial={{ width: 0 }}
                animate={{ width: 128 }}
                transition={{ duration: 0.3, ease: "easeOut" }}
              >
                <ProgressBar
                  progress={progress.progress}
                  size="sm"
                  color="secondary"
                  showPercentage={true}
                  animated={true}
                />
              </motion.div>
            ) : (
              <Button
                onClick={() => showVersionSelector(pack)}
                size="sm"
                variant="primary"
                icon={Download}
              >
                Install
              </Button>
            )}
          </div>
        </div>
      </motion.div>
    );
  };


  return (
    <div className="flex-1 p-6 text-white overflow-y-auto">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-white mb-2">Modpack Browser</h1>
          <p className="text-gray-300">Discover and install modpacks from Modrinth and CurseForge</p>
        </div>

        {/* Controls */}
        <div className="mb-6 space-y-4">
          {/* Search and Platform Selection */}
          <div className="flex gap-4">
            <form onSubmit={handleSearch} className="flex-1 flex gap-2">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                <input
                  type="text"
                  placeholder="Search modpacks..."
                  value={searchInput}
                  onChange={(e) => setSearchInput(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 bg-primary-800/90 border border-secondary-600/30 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:border-secondary-500/50"
                />
              </div>
              <Button
                type="submit"
                disabled={loading}
                loading={loading}
                icon={Search}
              >
                Search
              </Button>
            </form>
            
            {/* Platform Filter */}
            <div className="flex items-center gap-2">
              <Filter className="w-4 h-4 text-gray-400" />
              <select
                value={selectedPlatform}
                onChange={(e) => setSelectedPlatform(e.target.value as 'modrinth' | 'curseforge' | 'all')}
                className="px-3 py-2 bg-primary-800/90 border border-secondary-600/30 rounded-lg text-white focus:outline-none focus:border-secondary-500/50"
              >
                <option value="modrinth">Modrinth Only</option>
                <option value="curseforge">CurseForge Only</option>
                <option value="all">All Platforms</option>
              </select>
            </div>
          </div>

          {/* Action Buttons and View Controls */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Button
                onClick={openModpackFile}
                variant="secondary"
                icon={FileDown}
              >
                Import File
              </Button>
              <Button
                onClick={refresh}
                disabled={loading}
                variant="secondary"
                icon={RefreshCw}
              >
                Refresh
              </Button>
            </div>

            <div className="flex items-center gap-2">
              <motion.button
                onClick={() => setViewMode('grid')}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                transition={{ type: "spring", stiffness: 400, damping: 25 }}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'grid' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-gray-300 hover:bg-primary-600'
                }`}
              >
                <Grid className="w-4 h-4" />
              </motion.button>
              <motion.button
                onClick={() => setViewMode('list')}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                transition={{ type: "spring", stiffness: 400, damping: 25 }}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'list' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-gray-300 hover:bg-primary-600'
                }`}
              >
                <List className="w-4 h-4" />
              </motion.button>
            </div>
          </div>
        </div>

        {/* Error Message */}
        {(error || installError) && (
          <motion.div 
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.3, ease: "easeOut" }}
            className="mb-6 p-4 bg-red-900/50 border border-red-500/50 rounded-lg flex items-center gap-2"
          >
            <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
            <span className="text-red-200">{error || installError}</span>
            <motion.button
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.95 }}
              onClick={() => {
                setInstallError(null);
              }}
              className="ml-auto text-red-400 hover:text-red-300 transition-colors duration-200 p-1 rounded"
            >
              ×
            </motion.button>
          </motion.div>
        )}

        {/* Results */}
        <div className="mb-4">
          <h2 className="text-xl font-semibold text-white mb-4">
            {searchQuery ? `Search Results (${modpacks.length})` : 'Featured Modpacks'}
          </h2>
        </div>

        {modpacks.length === 0 && !loading ? (
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4, ease: "easeOut" }}
            className="text-center py-12"
          >
            <motion.div
              animate={{ 
                y: [0, -8, 0],
                rotate: [0, 5, -5, 0]
              }}
              transition={{ 
                duration: 2,
                repeat: Infinity,
                ease: "easeInOut"
              }}
            >
              <Package className="w-16 h-16 text-gray-500 mx-auto mb-4" />
            </motion.div>
            <h3 className="text-xl font-semibold text-gray-400 mb-2">
              {searchQuery ? 'No modpacks found' : 'No modpacks available'}
            </h3>
            <p className="text-gray-500">
              {searchQuery ? 'Try adjusting your search terms' : 'Featured modpacks will appear here'}
            </p>
          </motion.div>
        ) : (
          <>
            <div className={
              viewMode === 'grid'
                ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4'
                : 'space-y-3'
            }>
              {modpacks.map((pack, index) => renderModpackCard(pack, index))}
            </div>
            
            {/* Infinite scroll sentinel */}
            <div ref={sentinelRef} className="flex justify-center py-8">
              {loading && (
                <motion.div 
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.3, ease: "easeOut" }}
                  className="flex items-center gap-2 text-secondary-400"
                >
                  <motion.div
                    animate={{ rotate: 360 }}
                    transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
                  >
                    <Loader2 size={20} />
                  </motion.div>
                  <span>Loading more modpacks...</span>
                </motion.div>
              )}
              {!hasMore && modpacks.length > 0 && (
                <motion.div 
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ duration: 0.5, ease: "easeOut" }}
                  className="text-primary-400 text-sm"
                >
                  No more modpacks to load
                </motion.div>
              )}
            </div>
          </>
        )}

        {/* Version Selection Modal */}
        <Modal
          isOpen={showVersionModal && selectedPack !== null}
          onClose={() => setShowVersionModal(false)}
          title={`Select Version for ${selectedPack?.name || ''}`}
          size="lg"
        >
          <div className="p-6">
            <p className="text-sm text-gray-300 mb-4">
              Choose which version you'd like to install
            </p>
                {loadingVersions ? (
                  <div className="flex items-center justify-center py-8">
                    <Loader2 className="animate-spin w-6 h-6 text-secondary-400" />
                    <span className="ml-2 text-secondary-400">Loading versions...</span>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {availableVersions.map((version) => (
                      <div
                        key={version.id}
                        className="bg-primary-700 border border-secondary-600/30 rounded-lg p-4 hover:border-secondary-500/50 transition-colors"
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <h3 className="font-medium text-white">
                                {version.name}
                              </h3>
                              <span className="px-2 py-1 bg-secondary-600/50 text-xs text-secondary-300 rounded">
                                {version.version_type}
                              </span>
                            </div>
                            <div className="flex items-center gap-4 text-sm text-gray-300">
                              <span>Version: {version.version_number}</span>
                              <span>MC: {version.game_versions.join(', ')}</span>
                              <span>Downloads: {version.downloads.toLocaleString()}</span>
                            </div>
                            {version.loaders.length > 0 && (
                              <div className="flex items-center gap-2 mt-1">
                                <span className="text-xs text-gray-400">Loaders:</span>
                                {version.loaders.map((loader) => (
                                  <span
                                    key={loader}
                                    className="px-2 py-0.5 bg-primary-600 text-xs text-gray-300 rounded"
                                  >
                                    {loader}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                          <Button
                            onClick={() => selectedPack && installModpack(selectedPack, version)}
                            variant="primary"
                            icon={Download}
                          >
                            Install
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
        </Modal>
      </div>
    </div>
  );
}