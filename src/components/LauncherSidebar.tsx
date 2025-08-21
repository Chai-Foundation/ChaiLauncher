import React from 'react';
import { Home, Package, Settings, Plus, Download, User } from 'lucide-react';
import { motion } from 'framer-motion';

interface LauncherSidebarProps {
  activeView: string;
  onViewChange: (view: string) => void;
}

const LauncherSidebar: React.FC<LauncherSidebarProps> = ({ activeView, onViewChange }) => {
  const menuItems = [
    { id: 'home', label: 'Home', icon: Home },
    { id: 'instances', label: 'Instances', icon: Package },
    { id: 'browse', label: 'Browse Modpacks', icon: Plus },
    { id: 'downloads', label: 'Downloads', icon: Download },
    { id: 'accounts', label: 'Accounts', icon: User },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  return (
    <div className="w-64 bg-stone-900/30 backdrop-blur-sm border-r border-amber-600/30 flex flex-col">
      <div className="p-4 border-b border-amber-600/30">
        <h1 className="text-xl font-bold bg-gradient-to-r from-amber-200 via-white to-amber-200 bg-clip-text text-transparent">ChaiLauncher</h1>
        <p className="text-sm text-stone-300">Minecraft Launcher</p>
      </div>
      
      <nav className="flex-1 p-4">
        <ul className="space-y-2">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeView === item.id;
            
            return (
              <li key={item.id}>
                <button
                  onClick={() => onViewChange(item.id)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-300 relative ${
                    isActive 
                      ? 'bg-gradient-to-r from-amber-600 to-amber-500 text-white shadow-lg shadow-amber-500/25' 
                      : 'text-stone-200 hover:bg-stone-800/50 hover:text-white hover:border-amber-500/30 border border-transparent'
                  }`}
                >
                  {isActive && (
                    <motion.div
                      layoutId="activeTab"
                      className="absolute inset-0 bg-gradient-to-r from-amber-600 to-amber-500 rounded-lg"
                      initial={false}
                      transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                    />
                  )}
                  <Icon size={20} className="relative z-10" />
                  <span className="relative z-10">{item.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>
      
      <div className="p-4 border-t border-amber-600/30">
        <div className="text-sm text-stone-300">
          <p>Version 2.0.0</p>
          <p className="text-amber-300">Ready to launch</p>
        </div>
      </div>
    </div>
  );
};

export default LauncherSidebar;