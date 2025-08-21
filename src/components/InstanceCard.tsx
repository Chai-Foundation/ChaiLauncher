import React from 'react';
import { Play, Settings, Trash2, Clock, Package } from 'lucide-react';
import { motion } from 'framer-motion';
import { MinecraftInstance } from '../types/minecraft';

interface InstanceCardProps {
  instance: MinecraftInstance;
  onPlay: (instance: MinecraftInstance) => void;
  onEdit: (instance: MinecraftInstance) => void;
  onDelete: (instance: MinecraftInstance) => void;
}

const InstanceCard: React.FC<InstanceCardProps> = ({ instance, onPlay, onEdit, onDelete }) => {
  const formatPlayTime = (minutes: number) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours}h ${mins}m`;
  };

  const formatLastPlayed = (date?: Date) => {
    if (!date) return 'Never played';
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    return date.toLocaleDateString();
  };

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="bg-stone-900/50 backdrop-blur-sm rounded-xl border border-amber-600/30 overflow-hidden hover:border-amber-500/50 transition-all duration-300 hover:transform hover:scale-105 group"
    >
      <div className="aspect-video bg-gradient-to-br from-amber-600 to-amber-500 relative">
        {instance.icon ? (
          <img 
            src={instance.icon} 
            alt={instance.name} 
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Package size={48} className="text-white opacity-60" />
          </div>
        )}
        {instance.isExternal && (
          <div className="absolute top-2 left-2 bg-amber-600 text-white text-xs px-2 py-1 rounded-full">
            {instance.externalLauncher?.toUpperCase()}
          </div>
        )}
        <div className="absolute inset-0 bg-black bg-opacity-40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
          <button
            onClick={() => onPlay(instance)}
            className="bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-white p-2 rounded-lg transition-all duration-300 shadow-lg"
            title="Play"
          >
            <Play size={20} />
          </button>
          <button
            onClick={() => onEdit(instance)}
            className="bg-stone-700 hover:bg-stone-600 text-white p-2 rounded-lg transition-all duration-300"
            title="Edit"
          >
            <Settings size={20} />
          </button>
          <button
            onClick={() => onDelete(instance)}
            className="bg-red-600 hover:bg-red-700 text-white p-2 rounded-lg transition-colors"
            title="Delete"
          >
            <Trash2 size={20} />
          </button>
        </div>
      </div>
      
      <div className="p-4">
        <h3 className="font-semibold text-white mb-1">{instance.name}</h3>
        <p className="text-sm text-stone-300 mb-2">
          Minecraft {instance.version}
          {instance.modpack && ` • ${instance.modpack}`}
        </p>
        
        <div className="flex items-center justify-between text-xs text-stone-400">
          <div className="flex items-center gap-1">
            <Clock size={12} />
            <span>{formatLastPlayed(instance.lastPlayed)}</span>
          </div>
          <div className="flex items-center gap-4">
            {instance.isModded && (
              <span>{instance.modsCount} mods</span>
            )}
            <span>{formatPlayTime(instance.totalPlayTime)}</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
};

export default InstanceCard;