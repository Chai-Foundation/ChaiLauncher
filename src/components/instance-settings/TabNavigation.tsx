import React from 'react';
import { Settings, Package, Folder, Camera, Image } from 'lucide-react';

interface TabNavigationProps {
  activeTab: 'general' | 'jvm' | 'mods' | 'resourcepacks' | 'screenshots';
  onTabChange: (tab: 'general' | 'jvm' | 'mods' | 'resourcepacks' | 'screenshots') => void;
}

export const TabNavigation: React.FC<TabNavigationProps> = ({
  activeTab,
  onTabChange
}) => {
  const tabs = [
    { id: 'general', label: 'General', icon: Settings },
    { id: 'jvm', label: 'Java/JVM', icon: Package },
    { id: 'mods', label: 'Mods', icon: Folder },
    { id: 'resourcepacks', label: 'Resource Packs', icon: Image },
    { id: 'screenshots', label: 'Screenshots', icon: Camera }
  ] as const;

  return (
    <div className="flex border-b border-primary-700">
      {tabs.map((tab) => {
        const Icon = tab.icon;
        return (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors border-b-2 ${
              activeTab === tab.id
                ? 'border-secondary-500 text-white'
                : 'border-transparent text-primary-400 hover:text-white'
            }`}
          >
            <Icon size={18} />
            <span className="hidden sm:inline">{tab.label}</span>
          </button>
        );
      })}
    </div>
  );
};