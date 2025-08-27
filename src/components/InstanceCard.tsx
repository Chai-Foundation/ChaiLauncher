import React, { useRef, useEffect } from 'react';
import ReactDOM from 'react-dom';
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
      default: return 'border-secondary-600/30';
    }
  };

  const isDisabled = instance.status === 'installing' || instance.status === 'failed' || instance.status === 'invalid';

  // Context menu state
  const [showMenu, setShowMenu] = React.useState(false);
  const [menuPos, setMenuPos] = React.useState<{ x: number; y: number } | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const iconAreaRef = useRef<HTMLDivElement>(null);

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation(); // Prevent global handler from blocking
    setShowMenu(true);
    setMenuPos({ x: e.clientX, y: e.clientY });
  };

  // Trap focus and close on outside click/esc
  useEffect(() => {
    if (!showMenu) return;
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setShowMenu(false);
    };
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setShowMenu(false);
    };
    window.addEventListener('mousedown', handleClick);
    window.addEventListener('keydown', handleEsc);
    return () => {
      window.removeEventListener('mousedown', handleClick);
      window.removeEventListener('keydown', handleEsc);
    };
  }, [showMenu]);

  const [isHovered, setIsHovered] = React.useState(false);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className={`allow-context-menu bg-primary-900/50 backdrop-blur-sm rounded-xl border overflow-hidden transition-all duration-300 ${getStatusColor()} ${!isDisabled ? 'hover:border-secondary-500/50 hover:shadow-xl hover:scale-[1.03]' : ''}`}
      onContextMenu={handleContextMenu} // <-- Move here!
      tabIndex={0}
    >
      <div
        className="group aspect-video bg-gradient-to-br from-secondary-600 to-secondary-500 relative"
        ref={iconAreaRef}
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
            <Package size={48} className="text-white opacity-60" />
          </div>
        )}

        {/* Play icon overlay - only on hover */}
        {!isDisabled && isHovered && (
          <div className="absolute inset-0 pointer-events-none">
            {/* Dim overlay */}
            <div className="absolute inset-0 bg-black/40 opacity-100 transition-opacity duration-200 pointer-events-none" />
            {/* Animated Play icon */}
            <motion.div
              initial={{ scale: 0.8, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.8, opacity: 0 }}
              className="absolute inset-0 flex items-center justify-center opacity-100 transition-opacity duration-200 pointer-events-none"
              whileHover={{ scale: 1.15 }}
              transition={{ type: "spring", stiffness: 260, damping: 20 }}
            >
              <Play size={48} className="text-white drop-shadow-lg" />
            </motion.div>
            {/* Clickable area */}
            <button
              onClick={() => onPlay(instance)}
              className="absolute inset-0 w-full h-full cursor-pointer"
              aria-label="Play"
              tabIndex={-1}
              style={{ background: "transparent", border: "none", pointerEvents: "auto" }}
            />
          </div>
        )}

        {/* Status overlays */}
        {instance.status === 'installing' && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-gradient-to-br from-black/90 to-black/80 backdrop-blur-sm flex items-center justify-center flex-col"
            transition={{ duration: 0.4 }}
          >
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
                <span className="text-white text-lg font-bold drop-shadow">{Math.round(instance.installProgress || 0)}%</span>
              </div>
            </div>
            <div className="flex items-center gap-2 text-white mb-2">
              <Download size={18} className="animate-pulse text-blue-400" />
              <span className="text-sm font-medium">Installing...</span>
            </div>
          </motion.div>
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
          <div className="absolute top-2 left-2 bg-secondary-600 text-white text-xs px-2 py-1 rounded-full">
            {instance.externalLauncher?.toUpperCase()}
          </div>
        )}

        {/* Context menu (right-click) - uses portal for robustness */}
        {showMenu && menuPos &&
          ReactDOM.createPortal(
            <motion.div
              ref={menuRef}
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
              transition={{ duration: 0.18 }}
              className="fixed z-50 bg-primary-800 border border-primary-700 rounded-lg shadow-lg p-2 flex flex-col min-w-[140px]"
              style={{ left: menuPos.x, top: menuPos.y }}
              tabIndex={-1}
            >
              <button
                onClick={() => { onEdit(instance); setShowMenu(false); }}
                className="flex items-center gap-2 px-3 py-2 hover:bg-primary-700 rounded text-white text-sm"
                autoFocus
              >
                <Settings size={16} /> Edit
              </button>
              {onOpenFolder && (
                <button
                  onClick={() => { onOpenFolder(instance); setShowMenu(false); }}
                  className="flex items-center gap-2 px-3 py-2 hover:bg-blue-700 rounded text-white text-sm"
                >
                  <Folder size={16} /> Open Folder
                </button>
              )}
              <button
                onClick={() => { onDelete(instance); setShowMenu(false); }}
                className="flex items-center gap-2 px-3 py-2 hover:bg-red-700 rounded text-white text-sm"
              >
                <Trash2 size={16} /> Delete
              </button>
            </motion.div>,
            document.body
          )
        }
      </div>
      <div className="p-4">
        <h3 className="font-semibold text-white mb-1">{instance.name}</h3>
        {instance.status === 'failed' || instance.status === 'invalid' ? (
          <div className="mb-2">
            <p className="text-sm text-primary-300 mb-1">
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
            <p className="text-sm text-primary-300 mb-1">
              Minecraft {instance.version}
              {instance.modpack && ` • ${instance.modpack}`}
            </p>
            {instance.status === 'installing' && (
              <div className="flex items-center gap-2 mt-2">
                <div className="flex-1 bg-primary-700 rounded-full h-2 overflow-hidden">
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

        <div className="flex items-center justify-between text-xs text-primary-400">
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