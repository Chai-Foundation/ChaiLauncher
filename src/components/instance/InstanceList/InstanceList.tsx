import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { MinecraftInstance } from '../../../types/minecraft';
import InstanceCard from '../InstanceCard';

interface InstanceListProps {
  instances: MinecraftInstance[];
  onPlay: (instance: MinecraftInstance) => void;
  onEdit: (instance: MinecraftInstance) => void;
  onDelete: (instance: MinecraftInstance) => void;
  onOpenFolder?: (instance: MinecraftInstance) => void;
  viewMode?: 'grid' | 'list';
  loading?: boolean;
}

const InstanceList: React.FC<InstanceListProps> = ({
  instances,
  onPlay,
  onEdit,
  onDelete,
  onOpenFolder,
  viewMode = 'grid',
  loading = false
}) => {
  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {Array.from({ length: 6 }).map((_, i) => (
          <div
            key={i}
            className="bg-primary-900/50 backdrop-blur-sm rounded-xl border border-secondary-600/30 overflow-hidden"
          >
            <div className="aspect-video bg-primary-800 animate-pulse" />
            <div className="p-4 space-y-2">
              <div className="h-4 bg-primary-800 rounded animate-pulse" />
              <div className="h-3 bg-primary-800 rounded w-2/3 animate-pulse" />
              <div className="grid grid-cols-2 gap-3">
                <div className="h-3 bg-primary-800 rounded animate-pulse" />
                <div className="h-3 bg-primary-800 rounded animate-pulse" />
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (instances.length === 0) {
    return (
      <div className="text-center py-16">
        <div className="text-6xl mb-4">🎮</div>
        <h3 className="text-xl font-semibold text-white mb-2">No instances found</h3>
        <p className="text-primary-400">
          Create your first Minecraft instance to get started
        </p>
      </div>
    );
  }

  const containerClasses = viewMode === 'grid' 
    ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6'
    : 'space-y-4';

  return (
    <motion.div
      layout
      className={containerClasses}
    >
      <AnimatePresence mode="popLayout">
        {instances.map((instance) => (
          <InstanceCard
            key={instance.id}
            instance={instance}
            onPlay={onPlay}
            onEdit={onEdit}
            onDelete={onDelete}
            onOpenFolder={onOpenFolder}
          />
        ))}
      </AnimatePresence>
    </motion.div>
  );
};

export default InstanceList;