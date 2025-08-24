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
  AlertCircle,
  ExternalLink,
  User,
  Calendar,
  Tag
} from 'lucide-react';
import { ModInfo, ModSearchFilters } from '../types/mods';
import { MinecraftInstance } from '../types/minecraft';

interface ModsViewProps {
  selectedInstance?: MinecraftInstance | null;
}

export default function ModsView({ selectedInstance }: ModsViewProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<ModInfo[]>([]);
  const [featuredMods, setFeaturedMods] = useState<ModInfo[]>([]);
  const [categories, setCategories] = useState<string[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [loading, setLoading] = useState(false);
  const [featuredLoading, setFeaturedLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load initial data
  useEffect(() => {
    loadFeaturedMods();
    loadCategories();
  }, []);

  const loadFeaturedMods = async () => {
    setFeaturedLoading(true);
    try {
      const featured = await invoke<ModInfo[]>('get_featured_mods', {
        gameVersion: selectedInstance?.version,
        modLoader: null,
        limit: 12
      });
      setFeaturedMods(featured);
    } catch (err) {
      console.error('Failed to load featured mods:', err);
      setError('Failed to load featured mods');
    } finally {
      setFeaturedLoading(false);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await invoke<string[]>('get_mod_categories');
      setCategories(cats);
    } catch (err) {
      console.error('Failed to load categories:', err);
    }
  };

  const searchMods = async () => {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const results = await invoke<ModInfo[]>('search_mods', {
        query: searchQuery,
        gameVersion: selectedInstance?.version,
        modLoader: null,
        limit: 20
      });
      setSearchResults(results);
    } catch (err) {
      console.error('Search failed:', err);
      setError('Search failed. Please try again.');
      setSearchResults([]);
    } finally {
      setLoading(false);
    }
  };

  const installMod = async (mod: ModInfo) => {
    if (!selectedInstance) {
      setError('Please select an instance first');
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
      setError(`Failed to install ${mod.name}`);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    searchMods();
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

  const ModCard = ({ mod }: { mod: ModInfo }) => (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      className="bg-stone-800 border border-stone-700 rounded-lg p-4 hover:border-stone-600 transition-colors"
    >
      <div className="flex items-start gap-3">
        {mod.icon_url ? (
          <img
            src={mod.icon_url}
            alt={mod.name}
            className="w-12 h-12 rounded-lg object-cover"
          />
        ) : (
          <div className="w-12 h-12 bg-stone-700 rounded-lg flex items-center justify-center">
            <Package size={24} className="text-stone-400" />
          </div>
        )}
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-semibold text-white truncate">{mod.name}</h3>
            {mod.featured && (
              <Star size={16} className="text-yellow-500 fill-current" />
            )}
          </div>
          
          <p className="text-stone-400 text-sm mb-2 line-clamp-2">
            {mod.description}
          </p>
          
          <div className="flex items-center justify-between text-xs text-stone-500 mb-3">
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
                  className="px-2 py-1 bg-stone-700 text-stone-300 text-xs rounded"
                >
                  {category}
                </span>
              ))}
              {mod.categories.length > 2 && (
                <span className="px-2 py-1 bg-stone-700 text-stone-300 text-xs rounded">
                  +{mod.categories.length - 2}
                </span>
              )}
            </div>
            
            <div className="flex gap-2">
              {mod.website_url && (
                <button
                  onClick={() => window.open(mod.website_url, '_blank')}
                  className="p-1 text-stone-400 hover:text-white transition-colors"
                  title="Visit website"
                >
                  <ExternalLink size={16} />
                </button>
              )}
              <button
                onClick={() => installMod(mod)}
                disabled={!selectedInstance}
                className="px-3 py-1 bg-green-600 hover:bg-green-700 disabled:bg-stone-600 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
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
    <div className="flex-1 overflow-hidden bg-stone-900 text-white">
      <div className="h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-stone-700">
          <div>
            <h1 className="text-2xl font-bold text-white mb-1">Mods</h1>
            <p className="text-stone-400">
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
                viewMode === 'grid' ? 'bg-amber-600 text-white' : 'bg-stone-700 text-stone-300 hover:bg-stone-600'
              }`}
            >
              <Grid size={18} />
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'list' ? 'bg-amber-600 text-white' : 'bg-stone-700 text-stone-300 hover:bg-stone-600'
              }`}
            >
              <List size={18} />
            </button>
            <button
              onClick={loadFeaturedMods}
              className="p-2 bg-stone-700 text-stone-300 hover:bg-stone-600 rounded transition-colors"
            >
              <RefreshCw size={18} />
            </button>
          </div>
        </div>

        {/* Search Bar */}
        <div className="p-6 border-b border-stone-700">
          <form onSubmit={handleSearch} className="flex gap-4">
            <div className="flex-1 relative">
              <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-stone-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search mods..."
                className="w-full pl-10 pr-4 py-2 bg-stone-800 border border-stone-700 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
              />
            </div>
            
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-amber-500"
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
              className="px-4 py-2 bg-amber-600 hover:bg-amber-700 disabled:bg-stone-600 text-white rounded-lg transition-colors flex items-center gap-2"
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
          {error && (
            <div className="mb-4 p-4 bg-red-900/50 border border-red-700 rounded-lg flex items-center gap-2 text-red-200">
              <AlertCircle size={18} />
              {error}
            </div>
          )}

          {searchResults.length > 0 ? (
            <div>
              <h2 className="text-xl font-semibold mb-4">Search Results ({searchResults.length})</h2>
              <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1'}`}>
                {searchResults.map((mod) => (
                  <ModCard key={mod.id} mod={mod} />
                ))}
              </div>
            </div>
          ) : searchQuery.trim() && !loading ? (
            <div className="text-center py-8 text-stone-400">
              <Package size={48} className="mx-auto mb-2" />
              <p>No mods found for "{searchQuery}"</p>
            </div>
          ) : (
            <div>
              <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Star size={20} className="text-yellow-500" />
                Featured Mods
                {featuredLoading && <Loader size={16} className="animate-spin" />}
              </h2>
              
              {featuredMods.length > 0 ? (
                <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1'}`}>
                  {featuredMods.map((mod) => (
                    <ModCard key={mod.id} mod={mod} />
                  ))}
                </div>
              ) : !featuredLoading ? (
                <div className="text-center py-8 text-stone-400">
                  <Package size={48} className="mx-auto mb-2" />
                  <p>No featured mods available</p>
                </div>
              ) : null}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}