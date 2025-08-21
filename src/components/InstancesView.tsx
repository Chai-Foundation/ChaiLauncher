import React, { useState } from 'react';
import { Plus, Search, Grid, List, Play, Settings, Folder } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { MinecraftInstance } from '../types/minecraft';
import InstanceCard from './InstanceCard';

interface InstancesViewProps {
  instances: MinecraftInstance[];
  onCreateInstance: () => void;
  onPlayInstance: (instance: MinecraftInstance) => void;
  onEditInstance: (instance: MinecraftInstance) => void;
  onDeleteInstance: (instance: MinecraftInstance) => void;
  onOpenFolder?: (instance: MinecraftInstance) => void;
}

const InstancesView: React.FC<InstancesViewProps> = ({
  instances,
  onCreateInstance,
  onPlayInstance,
  onEditInstance,
  onDeleteInstance,
  onOpenFolder,
}) => {
  // Debug logging for instances prop changes
  React.useEffect(() => {
    const installingInstances = instances.filter(i => i.status === 'installing');
    if (installingInstances.length > 0) {
      console.log('InstancesView received installing instances:', 
        installingInstances.map(i => ({ id: i.id, name: i.name, progress: i.installProgress, status: i.status }))
      );
    }
  }, [instances]);

  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'name' | 'lastPlayed' | 'version'>('lastPlayed');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [filterModded, setFilterModded] = useState<'all' | 'modded' | 'vanilla'>('all');

  const filteredInstances = instances
    .filter(instance => {
      if (!searchTerm) return true;
      return instance.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
             instance.version.toLowerCase().includes(searchTerm.toLowerCase()) ||
             (instance.modpack && instance.modpack.toLowerCase().includes(searchTerm.toLowerCase()));
    })
    .filter(instance => {
      if (filterModded === 'all') return true;
      if (filterModded === 'modded') return instance.isModded;
      return !instance.isModded;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'version':
          return a.version.localeCompare(b.version);
        case 'lastPlayed':
        default:
          if (!a.lastPlayed && !b.lastPlayed) return 0;
          if (!a.lastPlayed) return 1;
          if (!b.lastPlayed) return -1;
          return b.lastPlayed.getTime() - a.lastPlayed.getTime();
      }
    });

  return (
    <div className="flex-1 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Instances</h1>
          <p className="text-gray-400">{instances.length} instances</p>
        </div>
        <button
          onClick={onCreateInstance}
          className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors"
        >
          <Plus size={18} />
          Create Instance
        </button>
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 mb-6">
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1 relative">
            <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
            <input
              type="text"
              placeholder="Search instances..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          
          <div className="flex gap-2">
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="lastPlayed">Last Played</option>
              <option value="name">Name</option>
              <option value="version">Version</option>
            </select>
            
            <select
              value={filterModded}
              onChange={(e) => setFilterModded(e.target.value as any)}
              className="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All</option>
              <option value="modded">Modded</option>
              <option value="vanilla">Vanilla</option>
            </select>
            
            <div className="flex bg-gray-700 rounded-lg border border-gray-600">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 rounded-l-lg transition-colors ${
                  viewMode === 'grid' ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white'
                }`}
              >
                <Grid size={18} />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 rounded-r-lg transition-colors ${
                  viewMode === 'list' ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white'
                }`}
              >
                <List size={18} />
              </button>
            </div>
          </div>
        </div>
      </div>

      <AnimatePresence mode="wait">
        {filteredInstances.length > 0 ? (
          <motion.div
            key={viewMode}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className={
              viewMode === 'grid'
                ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4'
                : 'space-y-3'
            }
          >
            {filteredInstances.map((instance) => (
              viewMode === 'grid' ? (
                <InstanceCard
                  key={instance.id}
                  instance={instance}
                  onPlay={onPlayInstance}
                  onEdit={onEditInstance}
                  onDelete={onDeleteInstance}
                  onOpenFolder={onOpenFolder}
                />
              ) : (
                <motion.div
                  key={instance.id}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  className="bg-gray-800 border border-gray-700 rounded-lg p-4 flex items-center gap-4 hover:border-gray-600 transition-colors group"
                >
                  <div className="w-16 h-16 bg-gradient-to-br from-green-600 to-blue-600 rounded-lg flex-shrink-0"></div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold text-white truncate">{instance.name}</h3>
                    <p className="text-sm text-gray-400">
                      Minecraft {instance.version}
                      {instance.modpack && ` • ${instance.modpack}`}
                    </p>
                    <p className="text-xs text-gray-500">
                      {instance.lastPlayed ? instance.lastPlayed.toLocaleDateString() : 'Never played'}
                    </p>
                  </div>
                  <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => onPlayInstance(instance)}
                      className="bg-green-600 hover:bg-green-700 text-white p-2 rounded-lg transition-colors"
                    >
                      <Play size={16} />
                    </button>
                    <button
                      onClick={() => onEditInstance(instance)}
                      className="bg-blue-600 hover:bg-blue-700 text-white p-2 rounded-lg transition-colors"
                    >
                      <Settings size={16} />
                    </button>
                    {onOpenFolder && (
                      <button
                        onClick={() => onOpenFolder(instance)}
                        className="bg-orange-600 hover:bg-orange-700 text-white p-2 rounded-lg transition-colors"
                      >
                        <Folder size={16} />
                      </button>
                    )}
                  </div>
                </motion.div>
              )
            ))}
          </motion.div>
        ) : (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center py-12"
          >
            <div className="text-gray-400 mb-4">
              {searchTerm || filterModded !== 'all' 
                ? 'No instances match your search criteria' 
                : 'No instances created yet'
              }
            </div>
            {(!searchTerm && filterModded === 'all') && (
              <button
                onClick={onCreateInstance}
                className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-lg transition-colors"
              >
                Create Your First Instance
              </button>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default InstancesView;