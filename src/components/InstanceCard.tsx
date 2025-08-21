import React from 'react';
import { Play, Settings, Trash2, Clock, Package, Folder, Download, AlertTriangle, X } from 'lucide-react';
import { motion } from 'framer-motion';
import { MinecraftInstance } from '../types/minecraft';

interface InstanceCardProps {
  instance: MinecraftInstance;
  onPlay: (instance: MinecraftInstance) => void;
  onEdit: (instance: MinecraftInstance) => void;
  onDelete: (instance: MinecraftInstance) => void;
  onOpenFolder?: (instance: MinecraftInstance) => void;
}

const InstanceCard: React.FC<InstanceCardProps> = React.memo(({ instance, onPlay, onEdit, onDelete, onOpenFolder }) => {
  // Debug logging for progress updates
  React.useEffect(() => {
    if (instance.status === 'installing') {
      console.log(`InstanceCard [${instance.name}]: status=${instance.status}, progress=${instance.installProgress}%, id=${instance.id}`);
    }
  }, [instance.status, instance.installProgress, instance.name, instance.id]);

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

  const getStatusColor = () => {
    switch (instance.status) {
      case 'installing': return 'border-blue-500/50';
      case 'failed': return 'border-red-500/50';
      case 'invalid': return 'border-red-600/50';
      default: return 'border-amber-600/30';
    }
  };

  const isDisabled = instance.status === 'installing' || instance.status === 'failed' || instance.status === 'invalid';

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className={`bg-stone-900/50 backdrop-blur-sm rounded-xl border overflow-hidden transition-all duration-300 group ${getStatusColor()} ${!isDisabled ? 'hover:border-amber-500/50 hover:transform hover:scale-105' : ''}`}
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
        
        {/* Status overlays */}
        {instance.status === 'installing' && (
          <div className="absolute inset-0 bg-gradient-to-br from-black/90 to-black/80 backdrop-blur-sm flex items-center justify-center flex-col">
            {/* Elegant circular progress */}
            <div className="relative w-24 h-24 mb-3">
              <svg className="w-24 h-24 transform -rotate-90" viewBox="0 0 100 100">
                {/* Background circle */}
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="rgba(255,255,255,0.1)"
                  strokeWidth="6"
                  fill="none"
                />
                {/* Progress circle */}
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="url(#progressGradient)"
                  strokeWidth="6"
                  fill="none"
                  strokeLinecap="round"
                  strokeDasharray={`${2 * Math.PI * 42}`}
                  strokeDashoffset={`${2 * Math.PI * 42 * (1 - (Math.min(100, Math.max(0, instance.installProgress || 0)) / 100))}`}
                  className="transition-all duration-500 ease-out"
                />
                <defs>
                  <linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#3b82f6" />
                    <stop offset="100%" stopColor="#06b6d4" />
                  </linearGradient>
                </defs>
              </svg>
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-white text-lg font-bold">{Math.round(instance.installProgress || 0)}%</span>
              </div>
            </div>
            
            {/* Status text */}
            <div className="flex items-center gap-2 text-white">
              <Download size={18} className="animate-pulse text-blue-400" />
              <span className="text-sm font-medium">Installing...</span>
            </div>
          </div>
        )}
        
        {instance.status === 'failed' && (
          <div className="absolute inset-0 bg-red-900 bg-opacity-80 flex items-center justify-center flex-col">
            <X size={32} className="text-red-300 mb-2" />
            <span className="text-red-300 text-sm">Installation Failed</span>
          </div>
        )}
        
        {instance.status === 'invalid' && (
          <div className="absolute inset-0 bg-red-900 bg-opacity-80 flex items-center justify-center flex-col">
            <AlertTriangle size={32} className="text-red-300 mb-2" />
            <span className="text-red-300 text-sm">Invalid Instance</span>
          </div>
        )}

        {instance.isExternal && (
          <div className="absolute top-2 left-2 bg-amber-600 text-white text-xs px-2 py-1 rounded-full">
            {instance.externalLauncher?.toUpperCase()}
          </div>
        )}
        
        {!isDisabled && (
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
            {onOpenFolder && (
              <button
                onClick={() => onOpenFolder(instance)}
                className="bg-blue-600 hover:bg-blue-700 text-white p-2 rounded-lg transition-colors"
                title="Open Folder"
              >
                <Folder size={20} />
              </button>
            )}
            <button
              onClick={() => onDelete(instance)}
              className="bg-red-600 hover:bg-red-700 text-white p-2 rounded-lg transition-colors"
              title="Delete"
            >
              <Trash2 size={20} />
            </button>
          </div>
        )}
      </div>
      
      <div className="p-4">
        <h3 className="font-semibold text-white mb-1">{instance.name}</h3>
        
        {instance.status === 'failed' || instance.status === 'invalid' ? (
          <div className="mb-2">
            <p className="text-sm text-stone-300 mb-1">
              Minecraft {instance.version}
              {instance.modpack && ` • ${instance.modpack}`}
            </p>
            <p className="text-sm text-red-400 bg-red-900/20 px-2 py-1 rounded">
              <AlertTriangle size={12} className="inline mr-1" />
              {instance.errorMessage || (instance.status === 'failed' ? 'Installation failed' : 'Invalid instance')}
            </p>
          </div>
        ) : (
          <div className="mb-2">
            <p className="text-sm text-stone-300 mb-1">
              Minecraft {instance.version}
              {instance.modpack && ` • ${instance.modpack}`}
            </p>
            {instance.status === 'installing' && (
              <div className="flex items-center gap-2 mt-2">
                <div className="flex-1 bg-stone-700 rounded-full h-2 overflow-hidden">
                  <div 
                    className="h-full bg-gradient-to-r from-blue-500 to-cyan-400 rounded-full transition-all duration-500 ease-out"
                    style={{ width: `${Math.min(100, Math.max(0, instance.installProgress || 0))}%` }}
                  />
                </div>
                <span className="text-xs text-blue-400 font-medium min-w-[3rem] text-right">
                  {Math.round(instance.installProgress || 0)}%
                </span>
              </div>
            )}
          </div>
        )}
        
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
}, (prevProps, nextProps) => {
  // Custom comparison for better performance
  const prev = prevProps.instance;
  const next = nextProps.instance;
  
  return (
    prev.id === next.id &&
    prev.name === next.name &&
    prev.status === next.status &&
    prev.installProgress === next.installProgress &&
    prev.version === next.version &&
    prev.modpack === next.modpack &&
    prev.lastPlayed?.getTime() === next.lastPlayed?.getTime() &&
    prev.totalPlayTime === next.totalPlayTime &&
    prev.isModded === next.isModded &&
    prev.modsCount === next.modsCount &&
    prev.errorMessage === next.errorMessage
  );
});

InstanceCard.displayName = 'InstanceCard';

export default InstanceCard;