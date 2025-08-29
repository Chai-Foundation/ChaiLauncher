import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
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
  Tag
} from 'lucide-react';
import { ModInfo, ModSearchFilters } from '../types/mods';
import { MinecraftInstance } from '../types/minecraft';
import { useInfiniteMods } from '../hooks/useInfiniteMods';
import { useInfiniteScroll } from '../hooks/useInfiniteScroll';

interface ModsViewProps {
  selectedInstance?: MinecraftInstance | null;
}

export default function ModsView({ selectedInstance }: ModsViewProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [categories, setCategories] = useState<string[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [installError, setInstallError] = useState<string | null>(null);
  
  const { mods, loading, hasMore, error, loadMore, refresh } = useInfiniteMods({
    searchQuery,
    gameVersion: selectedInstance?.version,
    selectedCategory,
    limit: 20
  });
  
  const { sentinelRef } = useInfiniteScroll({
    loading,
    hasMore,
    onLoadMore: loadMore,
  });

  // Load initial data
  useEffect(() => {
    loadCategories();
  }, []);

  const loadCategories = async () => {
    try {
      const cats = await invoke<string[]>('get_mod_categories');
      setCategories(cats);
    } catch (err) {
      console.error('Failed to load categories:', err);
    }
  };


  const installMod = async (mod: ModInfo) => {
    if (!selectedInstance) {
      setInstallError('Please select an instance first');
      return;
    }

    try {
      await invoke('install_mod', {
        instanceId: selectedInstance.id,
        modId: mod.id,
        versionId: null
      });
      // Success feedback would come from event listeners
    } catch (err) {
      console.error('Failed to install mod:', err);
      setInstallError(`Failed to install ${mod.name}`);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    // The hook will automatically refresh when searchQuery changes
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

  const ModCard = ({ mod, index }: { mod: ModInfo; index: number }) => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: (index % 20) * 0.05 }}
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
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-semibold text-white truncate">{mod.name}</h3>
            {mod.featured && (
              <Star size={16} className="text-yellow-500 fill-current" />
            )}
          </div>
          
          <p className="text-primary-400 text-sm mb-2 line-clamp-2">
            {mod.description}
          </p>
          
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
              {mod.categories.slice(0, 2).map((category) => (
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
              <button
                onClick={() => installMod(mod)}
                disabled={!selectedInstance}
                className="px-3 py-1 bg-green-600 hover:bg-green-700 disabled:bg-primary-600 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
              >
                Install
              </button>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );

  return (
    <div className="flex-1 overflow-hidden bg-primary-900 text-white">
      <div className="h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-primary-700">
          <div>
            <h1 className="text-2xl font-bold text-white mb-1">Mods</h1>
            <p className="text-primary-400">
              {selectedInstance ? (
                `Browse and install mods for ${selectedInstance.name}`
              ) : (
                'Select an instance to install mods'
              )}
            </p>
          </div>
          
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
            <button
              onClick={refresh}
              className="p-2 bg-primary-700 text-primary-300 hover:bg-primary-600 rounded transition-colors"
            >
              <RefreshCw size={18} />
            </button>
          </div>
        </div>

        {/* Search Bar */}
        <div className="p-6 border-b border-primary-700">
          <form onSubmit={handleSearch} className="flex gap-4">
            <div className="flex-1 relative">
              <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-primary-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search mods..."
                className="w-full pl-10 pr-4 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
              />
            </div>
            
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="px-3 py-2 bg-primary-800 border border-primary-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
            >
              <option value="">All Categories</option>
              {categories.map((category) => (
                <option key={category} value={category}>
                  {category}
                </option>
              ))}
            </select>
            
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
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-6">
          {(error || installError) && (
            <div className="mb-4 p-4 bg-red-900/50 border border-red-700 rounded-lg flex items-center gap-2 text-red-200">
              <AlertCircle size={18} />
              {error || installError}
              <button
                onClick={() => {
                  setInstallError(null);
                }}
                className="ml-auto text-red-400 hover:text-red-300"
              >
                ×
              </button>
            </div>
          )}

          <div>
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
              {searchQuery.trim() ? (
                <>Search Results ({mods.length})</>
              ) : (
                <>
                  <Star size={20} className="text-yellow-500" />
                  {selectedCategory ? `${selectedCategory} Mods` : 'Featured Mods'}
                </>
              )}
            </h2>
            
            {mods.length > 0 ? (
              <>
                <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1'}`}>
                  {mods.map((mod, index) => (
                    <ModCard key={mod.id} mod={mod} index={index} />
                  ))}
                </div>
                
                {/* Infinite scroll sentinel */}
                <div ref={sentinelRef} className="flex justify-center py-8">
                  {loading && (
                    <div className="flex items-center gap-2 text-primary-400">
                      <Loader2 className="animate-spin" size={20} />
                      <span>Loading more mods...</span>
                    </div>
                  )}
                  {!hasMore && mods.length > 0 && (
                    <div className="text-primary-400 text-sm">
                      No more mods to load
                    </div>
                  )}
                </div>
              </>
            ) : !loading ? (
              <div className="text-center py-8 text-primary-400">
                <Package size={48} className="mx-auto mb-2" />
                <p>
                  {searchQuery.trim() ? `No mods found for "${searchQuery}"` : 'No mods available'}
                </p>
              </div>
            ) : (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="animate-spin" size={32} />
                <span className="ml-3 text-primary-300">Loading mods...</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}