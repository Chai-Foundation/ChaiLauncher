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
  AlertCircle,
  ExternalLink,
  User,
  Calendar,
  Tag,
  FileDown,
  Upload,
  FolderOpen
} from 'lucide-react';
import { ModrinthPack, ModpackInstallProgress, LauncherSettings } from '../types';

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
  const [searchResults, setSearchResults] = useState<ModrinthPack[]>([]);
  const [featuredPacks, setFeaturedPacks] = useState<ModrinthPack[]>([]);
  const [selectedPlatform, setSelectedPlatform] = useState<string>('modrinth');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [loading, setLoading] = useState(false);
  const [featuredLoading, setFeaturedLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installProgress, setInstallProgress] = useState<Map<string, ModpackInstallProgress>>(new Map());

  // Load initial data
  useEffect(() => {
    loadFeaturedPacks();
  }, [selectedPlatform]);

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

  const loadFeaturedPacks = async () => {
    setFeaturedLoading(true);
    setError(null);
    try {
      const featured = await invoke<ModrinthPack[]>('search_modpacks', {
        query: 'featured',
        platform: selectedPlatform,
        limit: 12
      });
      setFeaturedPacks(featured);
    } catch (error) {
      console.error('Failed to load featured modpacks:', error);
      setError('Failed to load featured modpacks');
    } finally {
      setFeaturedLoading(false);
    }
  };

  const searchModpacks = async () => {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const results = await invoke<ModrinthPack[]>('search_modpacks', {
        query: searchQuery,
        platform: selectedPlatform,
        limit: 20
      });
      setSearchResults(results);
    } catch (error) {
      console.error('Failed to search modpacks:', error);
      setError('Failed to search modpacks');
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    searchModpacks();
  };

  const installModpack = async (pack: ModrinthPack) => {
    if (!launcherSettings) {
      setError('Launcher settings not available');
      return;
    }

    try {
      const instanceDir = `${launcherSettings.instances_dir}/${pack.name}`;
      
      // Start installation
      await invoke('install_modpack', {
        instanceDir,
        platform: selectedPlatform,
        projectId: pack.project_id,
        versionId: pack.version_id
      });

      // Optionally trigger instance creation in parent component
      if (onCreateInstance) {
        onCreateInstance({
          name: pack.name,
          version: pack.game_versions[0] || '1.20.1',
          modpack: pack.name,
          modpackVersion: pack.version_id
        });
      }

    } catch (error) {
      console.error('Failed to install modpack:', error);
      setError(`Failed to install ${pack.name}: ${error}`);
    }
  };

  const openModpackFile = async () => {
    try {
      // This would open a file dialog to select modpack files
      // For now, show a message about importing
      setError('File import functionality coming soon!');
    } catch (error) {
      console.error('Failed to open file dialog:', error);
    }
  };

  const renderModpackCard = (pack: ModrinthPack, index: number) => {
    const progress = installProgress.get(`${launcherSettings?.instances_dir}/${pack.name}`);
    
    return (
      <motion.div
        key={pack.project_id}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: index * 0.1 }}
        className={`bg-primary-800/90 backdrop-blur-sm border border-secondary-600/30 rounded-lg overflow-hidden hover:border-secondary-500/50 transition-all duration-200 ${
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
              <div className="flex items-center gap-2">
                <div className="w-24 h-2 bg-primary-700 rounded-full overflow-hidden">
                  <div 
                    className="h-full bg-secondary-500 transition-all duration-300"
                    style={{ width: `${progress.progress}%` }}
                  />
                </div>
                <span className="text-xs text-secondary-400">{Math.round(progress.progress)}%</span>
              </div>
            ) : (
              <button
                onClick={() => installModpack(pack)}
                className="flex items-center gap-1 px-3 py-1 bg-secondary-600 hover:bg-secondary-500 text-white rounded text-sm transition-colors"
              >
                <Download className="w-3 h-3" />
                Install
              </button>
            )}
          </div>
        </div>
      </motion.div>
    );
  };

  const displayPacks = searchQuery ? searchResults : featuredPacks;
  const isLoading = loading || featuredLoading;

  return (
    <div className="flex-1 p-6 text-white overflow-y-auto">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-white mb-2">Modpack Browser</h1>
          <p className="text-gray-300">Discover and install modpacks from various platforms</p>
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
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 bg-primary-800/90 border border-secondary-600/30 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:border-secondary-500/50"
                />
              </div>
              <button
                type="submit"
                disabled={isLoading}
                className="px-4 py-2 bg-secondary-600 hover:bg-secondary-500 disabled:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2"
              >
                {isLoading ? <Loader className="w-4 h-4 animate-spin" /> : <Search className="w-4 h-4" />}
                Search
              </button>
            </form>

            <select
              value={selectedPlatform}
              onChange={(e) => setSelectedPlatform(e.target.value)}
              className="px-3 py-2 bg-primary-800/90 border border-secondary-600/30 rounded-lg text-white focus:outline-none focus:border-secondary-500/50"
            >
              <option value="modrinth">Modrinth</option>
              <option value="curseforge">CurseForge</option>
            </select>
          </div>

          {/* Action Buttons and View Controls */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <button
                onClick={openModpackFile}
                className="flex items-center gap-2 px-3 py-2 bg-primary-700 hover:bg-primary-600 text-white rounded-lg transition-colors"
              >
                <FileDown className="w-4 h-4" />
                Import File
              </button>
              <button
                onClick={loadFeaturedPacks}
                disabled={isLoading}
                className="flex items-center gap-2 px-3 py-2 bg-primary-700 hover:bg-primary-600 disabled:bg-primary-600 text-white rounded-lg transition-colors"
              >
                <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
                Refresh
              </button>
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'grid' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-gray-300 hover:bg-primary-600'
                }`}
              >
                <Grid className="w-4 h-4" />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'list' ? 'bg-secondary-600 text-white' : 'bg-primary-700 text-gray-300 hover:bg-primary-600'
                }`}
              >
                <List className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-6 p-4 bg-red-900/50 border border-red-500/50 rounded-lg flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
            <span className="text-red-200">{error}</span>
            <button
              onClick={() => setError(null)}
              className="ml-auto text-red-400 hover:text-red-300"
            >
              ×
            </button>
          </div>
        )}

        {/* Results */}
        <div className="mb-4">
          <h2 className="text-xl font-semibold text-white mb-4">
            {searchQuery ? `Search Results (${displayPacks.length})` : 'Featured Modpacks'}
          </h2>
        </div>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader className="w-8 h-8 animate-spin text-secondary-500" />
            <span className="ml-3 text-gray-300">Loading modpacks...</span>
          </div>
        ) : displayPacks.length === 0 ? (
          <div className="text-center py-12">
            <Package className="w-16 h-16 text-gray-500 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-gray-400 mb-2">
              {searchQuery ? 'No modpacks found' : 'No modpacks available'}
            </h3>
            <p className="text-gray-500">
              {searchQuery ? 'Try adjusting your search terms' : 'Featured modpacks will appear here'}
            </p>
          </div>
        ) : (
          <div className={
            viewMode === 'grid'
              ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4'
              : 'space-y-3'
          }>
            {displayPacks.map((pack, index) => renderModpackCard(pack, index))}
          </div>
        )}
      </div>
    </div>
  );
}