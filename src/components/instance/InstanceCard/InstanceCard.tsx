import React from 'react';
import { Play, Settings, Trash2, Clock, Package, Folder, Download, AlertTriangle, X, Square } from 'lucide-react';
import { motion } from 'framer-motion';
import { MinecraftInstance } from '../../../types/minecraft';
import { ContextMenu, ContextMenuItem, ProgressBar } from '../../ui';

interface InstanceCardProps {
  instance: MinecraftInstance;
  onPlay: (instance: MinecraftInstance) => void;
  onStop?: (instance: MinecraftInstance) => void;
  onEdit: (instance: MinecraftInstance) => void;
  onDelete: (instance: MinecraftInstance) => void;
  onOpenFolder?: (instance: MinecraftInstance) => void;
  viewMode?: 'grid' | 'list';
}

const InstanceCard = React.memo(React.forwardRef<HTMLDivElement, InstanceCardProps>(({ 
  instance, 
  onPlay, 
  onStop,
  onEdit, 
  onDelete, 
  onOpenFolder,
  viewMode = 'grid'
}, ref) => {
  const [showMenu, setShowMenu] = React.useState(false);
  const [menuPos, setMenuPos] = React.useState<{ x: number; y: number } | null>(null);
  const [isHovered, setIsHovered] = React.useState(false);

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
      case 'installing': return 'border-secondary-500/50';
      case 'failed': return 'border-secondary-500/50';
      case 'invalid': return 'border-secondary-600/50';
      case 'running': return 'border-green-500/50';
      default: return 'border-secondary-600/30';
    }
  };

  const isDisabled = instance.status === 'installing' || instance.status === 'failed' || instance.status === 'invalid';
  const isRunning = instance.status === 'running';

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setShowMenu(true);
    setMenuPos({ x: e.clientX, y: e.clientY });
  };

  const contextMenuItems: ContextMenuItem[] = [
    ...(isRunning && onStop ? [{
      label: 'Stop',
      icon: Square,
      onClick: () => onStop(instance)
    }] : [{
      label: 'Play',
      icon: Play,
      onClick: () => onPlay(instance),
      disabled: isDisabled
    }]),
    {
      label: 'Settings',
      icon: Settings,
      onClick: () => onEdit(instance)
    },
    ...(onOpenFolder ? [{
      label: 'Open Folder',
      icon: Folder,
      onClick: () => onOpenFolder(instance)
    }] : []),
    {
      label: 'Delete',
      icon: Trash2,
      onClick: () => onDelete(instance),
      danger: true
    }
  ];

  const renderStatusOverlay = () => {
    if (instance.status === 'running') {
      return (
        <div className="absolute top-2 right-2">
          <div className="flex items-center gap-1 bg-green-600/90 text-white px-2 py-1 rounded-full text-xs backdrop-blur-sm">
            <div className="w-2 h-2 bg-green-300 rounded-full animate-pulse" />
            Running
          </div>
        </div>
      );
    }

    if (instance.status === 'installing') {
      const progressSize = viewMode === 'list' ? 16 : 24;
      const iconSize = viewMode === 'list' ? 12 : 18;
      
      return (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="absolute inset-0 bg-gradient-to-br from-black/90 to-black/80 backdrop-blur-sm flex items-center justify-center flex-col"
        >
          {viewMode === 'grid' ? (
            <div className="relative w-24 h-24 mb-3 flex items-center justify-center">
              <svg className="w-24 h-24 transform -rotate-90" viewBox="0 0 100 100">
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="rgba(255,255,255,0.08)"
                  strokeWidth="7"
                  fill="none"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="url(#progressGradient)"
                  strokeWidth="7"
                  fill="none"
                  strokeLinecap="round"
                  strokeDasharray={`${2 * Math.PI * 42}`}
                  strokeDashoffset={`${2 * Math.PI * 42 * (1 - (Math.min(100, Math.max(0, instance.installProgress || 0)) / 100))}`}
                  className="transition-all duration-500 ease-out"
                />
                <defs>
                  <linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="var(--secondary-400)" />
                    <stop offset="100%" stopColor="var(--secondary-600)" />
                  </linearGradient>
                </defs>
              </svg>
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-white text-lg font-bold drop-shadow">
                  {Math.round(instance.installProgress || 0)}%
                </span>
              </div>
            </div>
          ) : (
            <div className="relative w-12 h-12 mb-1 flex items-center justify-center">
              <svg className="w-12 h-12 transform -rotate-90" viewBox="0 0 100 100">
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="rgba(255,255,255,0.08)"
                  strokeWidth="10"
                  fill="none"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="42"
                  stroke="url(#progressGradientSmall)"
                  strokeWidth="10"
                  fill="none"
                  strokeLinecap="round"
                  strokeDasharray={`${2 * Math.PI * 42}`}
                  strokeDashoffset={`${2 * Math.PI * 42 * (1 - (Math.min(100, Math.max(0, instance.installProgress || 0)) / 100))}`}
                  className="transition-all duration-500 ease-out"
                />
                <defs>
                  <linearGradient id="progressGradientSmall" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="var(--secondary-400)" />
                    <stop offset="100%" stopColor="var(--secondary-600)" />
                  </linearGradient>
                </defs>
              </svg>
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-white text-xs font-bold drop-shadow">
                  {Math.round(instance.installProgress || 0)}%
                </span>
              </div>
            </div>
          )}
          <div className="flex items-center gap-2 text-white mb-2">
            <Download size={iconSize} className="animate-pulse text-secondary-400" />
            <span className={`${viewMode === 'list' ? 'text-xs' : 'text-sm'} font-medium`}>Installing...</span>
          </div>
        </motion.div>
      );
    }

    if (instance.status === 'failed') {
      const iconSize = viewMode === 'list' ? 20 : 32;
      return (
        <div className="absolute inset-0 bg-secondary-900/80 flex items-center justify-center flex-col">
          <X size={iconSize} className="text-secondary-300 mb-2" />
          <span className={`text-secondary-300 ${viewMode === 'list' ? 'text-xs' : 'text-sm'}`}>Installation Failed</span>
        </div>
      );
    }

    if (instance.status === 'invalid') {
      const iconSize = viewMode === 'list' ? 20 : 32;
      return (
        <div className="absolute inset-0 bg-secondary-900/80 flex items-center justify-center flex-col">
          <AlertTriangle size={iconSize} className="text-secondary-300 mb-2" />
          <span className={`text-secondary-300 ${viewMode === 'list' ? 'text-xs' : 'text-sm'}`}>Invalid Instance</span>
        </div>
      );
    }

    return null;
  };

  return (
    <>
      <motion.div
        ref={ref}
        layout
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className={`allow-context-menu bg-primary-900/50 backdrop-blur-sm rounded-xl border overflow-hidden transition-all duration-300 ${getStatusColor()} ${!isDisabled ? 'hover:border-secondary-500/50 hover:shadow-xl' + (viewMode === 'grid' ? ' hover:scale-[1.03]' : '') : ''} ${
          viewMode === 'list' ? 'flex' : ''
        }`}
        onContextMenu={handleContextMenu}
        tabIndex={0}
      >
        <div
          className={`group bg-gradient-to-br from-secondary-600 to-secondary-500 relative ${
            viewMode === 'list' ? 'w-20 h-20 flex-shrink-0' : 'aspect-video'
          }`}
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
        >
          {instance.icon ? (
            <img 
              src={instance.icon} 
              alt={instance.name} 
              className="w-full h-full object-cover"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center">
              <Package size={viewMode === 'list' ? 32 : 48} className="text-white opacity-60" />
            </div>
          )}

          {/* Play/Stop icon overlay - only on hover */}
          {!isDisabled && isHovered && (
            <div className="absolute inset-0 pointer-events-none">
              <div className="absolute inset-0 bg-black/40 opacity-100 transition-opacity duration-200 pointer-events-none" />
              <motion.div
                initial={{ scale: 0.8, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                exit={{ scale: 0.8, opacity: 0 }}
                className="absolute inset-0 flex items-center justify-center opacity-100 transition-opacity duration-200 pointer-events-none"
                whileHover={{ scale: 1.15 }}
                transition={{ type: "spring", stiffness: 260, damping: 20 }}
              >
                {isRunning ? (
                  <Square size={48} className="text-red-400 drop-shadow-lg" />
                ) : (
                  <Play size={48} className="text-white drop-shadow-lg" />
                )}
              </motion.div>
              <button
                onClick={() => isRunning && onStop ? onStop(instance) : onPlay(instance)}
                className="absolute inset-0 w-full h-full cursor-pointer"
                aria-label={isRunning ? "Stop" : "Play"}
                tabIndex={-1}
                style={{ background: "transparent", border: "none", pointerEvents: "auto" }}
              />
            </div>
          )}

          {renderStatusOverlay()}
        </div>

        {/* Instance info */}
        <div className="p-4 flex-1">
          <div className="flex items-start justify-between mb-2">
            <div className="flex-1 min-w-0">
              <h3 className="text-white font-medium truncate text-base">
                {instance.name}
              </h3>
              <p className="text-primary-300 text-sm truncate">
                Minecraft {instance.version}
                {instance.modpack && (
                  <span className="text-secondary-400 ml-1">
                    • {instance.modpack}
                  </span>
                )}
              </p>
            </div>
            {instance.isExternal && (
              <span className="text-xs bg-primary-700 text-primary-300 px-2 py-1 rounded-full ml-2 flex-shrink-0">
                {instance.externalLauncher}
              </span>
            )}
          </div>

          <div className={`text-xs text-primary-400 ${
            viewMode === 'list' ? 'flex items-center gap-4' : 'grid grid-cols-2 gap-3'
          }`}>
            <div className="flex items-center gap-1">
              <Clock size={12} />
              <span>{formatLastPlayed(instance.lastPlayed)}</span>
            </div>
            <div className="flex items-center gap-1">
              <Package size={12} />
              <span>{instance.isModded ? `${instance.modsCount} mods` : 'Vanilla'}</span>
            </div>
          </div>

          {instance.errorMessage && (
            <div className="mt-2 p-2 bg-red-900/20 border border-red-700/50 rounded text-red-400 text-xs">
              {instance.errorMessage}
            </div>
          )}
        </div>
      </motion.div>

      <ContextMenu
        isOpen={showMenu}
        position={menuPos}
        items={contextMenuItems}
        onClose={() => setShowMenu(false)}
      />
    </>
  );
}));

InstanceCard.displayName = 'InstanceCard';

export default InstanceCard;