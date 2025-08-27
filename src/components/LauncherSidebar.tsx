import React, { CSSProperties, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Home, Package, Settings, Plus, Download, User, Server } from 'lucide-react';
import { motion } from 'framer-motion';

interface LauncherSidebarProps {
  activeView: string;
  onViewChange: (view: string) => void;
}

const LauncherSidebar: React.FC<LauncherSidebarProps> = ({ activeView, onViewChange }) => {
  const menuItems = [
    { id: 'home', label: 'Home', icon: Home },
    { id: 'instances', label: 'Instances', icon: Package },
    { id: 'servers', label: 'Servers', icon: Server },
    { id: 'browse', label: 'Browse Modpacks', icon: Plus },
    { id: 'downloads', label: 'Downloads', icon: Download },
    { id: 'accounts', label: 'Accounts', icon: User },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  const [appVersion, setAppVersion] = useState<string>("");

  useEffect(() => {
    invoke<string>("get_app_version")
      .then(setAppVersion)
      .catch(() => setAppVersion(""));
  }, []);

  return (
    <div className="w-64 bg-primary-900/60 backdrop-blur-sm flex flex-col" style={{ WebkitAppRegion: 'drag' } as CSSProperties}>
      <div className="flex flex-row items-center p-2">
        <img src="Square310x310Logo.png" className="w-16 h-16 mr-3" alt="ChaiLauncher Logo" />
        <div>
          <h1 className="text-xl font-bold bg-gradient-to-r from-amber-200 via-white to-amber-200 bg-clip-text text-transparent">
        ChaiLauncher
          </h1>
          <p className="text-sm text-primary-300">Minecraft Launcher</p>
        </div>
      </div>
      
      <nav className="flex-1 p-4">
      <ul className="space-y-2">
        {menuItems.map((item) => {
        const Icon = item.icon;
        const isActive = activeView === item.id;
        
        return (
          <li key={item.id}>
          {/* @ts-ignore */}
          <button
            ref={el => {
            if (el) el.setAttribute('style', 'WebkitAppRegion: no-drag;');
            }}
            onClick={() => onViewChange(item.id)}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-300 relative ${
            isActive 
              ? 'bg-gradient-to-r from-amber-600 to-amber-500 text-white shadow-lg shadow-amber-500/25' 
              : 'text-primary-200 hover:bg-primary-800/50 hover:text-white hover:border-secondary-500/30 border border-transparent'
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
      
      <div className="p-4 border-t border-secondary-600/30">
        <div className="text-sm text-primary-300">
          <p>Version {appVersion || "..."}</p>
          <p className="text-secondary-300">Ready to launch</p>
        </div>
      </div>
    </div>
  );
};

export default LauncherSidebar;