import React, { useState } from 'react';
import { Plus, Search, Grid, List, Play, Settings, Folder, Package } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { MinecraftInstance } from '../types/minecraft';
import { InstanceList } from './instance';
import ModpackCreator from './ModpackCreator';
import { Button } from './ui';

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
  const [showModpackCreator, setShowModpackCreator] = useState(false);

  // Remove local modal state for editing
  // const [showInstanceSettings, setShowInstanceSettings] = useState(false);
  // const [selectedInstance, setSelectedInstance] = useState<MinecraftInstance | null>(null);

  // Use only the global edit handler
  const handleEditInstance = (instance: MinecraftInstance) => {
    onEditInstance(instance);
  };

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
          <p className="text-primary-400">{instances.length} instances</p>
        </div>
        <div className="flex gap-3">
          <Button
            onClick={() => setShowModpackCreator(true)}
            disabled={instances.length === 0}
            icon={Package}
          >
            Create Modpack
          </Button>
          <Button
            onClick={onCreateInstance}
            icon={Plus}
          >
            Create Instance
          </Button>
        </div>
      </div>

      <div className="bg-primary-800 border border-primary-700 rounded-lg p-4 mb-6">
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1 relative">
            <Search size={18} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-primary-400" />
            <input
              type="text"
              placeholder="Search instances..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-primary-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
            />
          </div>
          
          <div className="flex gap-2">
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="bg-primary-700 border border-primary-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
            >
              <option value="lastPlayed">Last Played</option>
              <option value="name">Name</option>
              <option value="version">Version</option>
            </select>
            
            <select
              value={filterModded}
              onChange={(e) => setFilterModded(e.target.value as any)}
              className="bg-primary-700 border border-primary-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-secondary-500"
            >
              <option value="all">All</option>
              <option value="modded">Modded</option>
              <option value="vanilla">Vanilla</option>
            </select>
            
            <div className="flex bg-primary-700 rounded-lg border border-primary-600">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 rounded-l-lg transition-colors ${
                  viewMode === 'grid' ? 'bg-secondary-600 text-white' : 'text-primary-400 hover:text-white'
                }`}
              >
                <Grid size={18} />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 rounded-r-lg transition-colors ${
                  viewMode === 'list' ? 'bg-secondary-600 text-white' : 'text-primary-400 hover:text-white'
                }`}
              >
                <List size={18} />
              </button>
            </div>
          </div>
        </div>
      </div>

      <InstanceList
        instances={filteredInstances}
        onPlay={onPlayInstance}
        onEdit={handleEditInstance}
        onDelete={onDeleteInstance}
        onOpenFolder={onOpenFolder}
        viewMode={viewMode}
      />

      {/* Modpack Creator Modal */}
      {showModpackCreator && (
        <ModpackCreator
          instances={instances.filter(instance => instance.status === 'ready')}
          onClose={() => setShowModpackCreator(false)}
          onCreateSuccess={(modpackPath) => {
            console.log('Modpack created at:', modpackPath);
            setShowModpackCreator(false);
          }}
        />
      )}
    </div>
  );
};

export default InstancesView;