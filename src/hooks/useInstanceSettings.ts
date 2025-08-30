import { useState, useEffect } from 'react';
import { MinecraftInstance } from '../types/minecraft';
import { ModInfo } from '../types/mods';
import { invoke } from '@tauri-apps/api/core';

interface ScreenshotInfo {
  id: string;
  filename: string;
  path: string;
  timestamp: Date;
  size: number;
}

export const useInstanceSettings = (instance: MinecraftInstance, isOpen: boolean) => {
  const [activeTab, setActiveTab] = useState<'general' | 'jvm' | 'mods' | 'resourcepacks' | 'screenshots'>('general');
  
  // Mod management state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<ModInfo[]>([]);
  const [installedMods, setInstalledMods] = useState<ModInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [hasMoreResults, setHasMoreResults] = useState(false);
  const [currentOffset, setCurrentOffset] = useState(0);
  const [isLoadingMore, setIsLoadingMore] = useState(false);

  // JVM settings state
  const [jvmSettings, setJvmSettings] = useState({
    javaPath: instance.javaPath || '',
    jvmArgs: instance.jvmArgs || [],
    minMemory: 2048, // MB
    maxMemory: 4096, // MB
    useCustomJava: !!instance.javaPath,
    useCustomArgs: (instance.jvmArgs && instance.jvmArgs.length > 0) || false
  });

  // Screenshots state
  const [screenshots, setScreenshots] = useState<ScreenshotInfo[]>([]);
  const [selectedScreenshot, setSelectedScreenshot] = useState<ScreenshotInfo | null>(null);
  const [screenshotLoading, setScreenshotLoading] = useState(false);

  // Resource packs state
  const [installedResourcePacks, setInstalledResourcePacks] = useState<string[]>([]);
  const [resourcePackLoading, setResourcePackLoading] = useState(false);

  // Load data based on active tab
  useEffect(() => {
    if (isOpen) {
      switch (activeTab) {
        case 'mods':
          loadInstalledMods();
          break;
        case 'screenshots':
          loadScreenshots();
          break;
        case 'resourcepacks':
          loadResourcePacks();
          break;
      }
    }
  }, [isOpen, activeTab, instance.id]);

  const loadInstalledMods = async () => {
    try {
      const mods = await invoke<ModInfo[]>('get_installed_mods', {
        instanceId: instance.id
      });
      setInstalledMods(mods);
    } catch (err) {
      console.error('Failed to load installed mods:', err);
      setInstalledMods([]);
    }
  };

  const loadScreenshots = async () => {
    setScreenshotLoading(true);
    try {
      const screenshots = await invoke<ScreenshotInfo[]>('get_instance_screenshots', {
        instanceId: instance.id
      });
      setScreenshots(screenshots);
    } catch (err) {
      console.error('Failed to load screenshots:', err);
      setScreenshots([]);
    } finally {
      setScreenshotLoading(false);
    }
  };

  const loadResourcePacks = async () => {
    setResourcePackLoading(true);
    try {
      const resourcePacks = await invoke<string[]>('get_instance_resource_packs', {
        instanceId: instance.id
      });
      setInstalledResourcePacks(resourcePacks);
    } catch (err) {
      console.error('Failed to load resource packs:', err);
      setInstalledResourcePacks([]);
    } finally {
      setResourcePackLoading(false);
    }
  };

  const searchMods = async (query: string, offset = 0) => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const results = await invoke<{ mods: ModInfo[]; hasMore: boolean }>('search_mods', {
        query: query.trim(),
        offset,
        limit: 20
      });

      if (offset === 0) {
        setSearchResults(results.mods);
      } else {
        setSearchResults(prev => [...prev, ...results.mods]);
      }

      setHasMoreResults(results.hasMore);
      setCurrentOffset(offset);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to search mods');
      setSearchResults([]);
    } finally {
      setLoading(false);
      setIsLoadingMore(false);
    }
  };

  const loadMoreMods = async () => {
    if (!hasMoreResults || isLoadingMore) return;
    
    setIsLoadingMore(true);
    const newOffset = currentOffset + 20;
    await searchMods(searchQuery, newOffset);
  };

  const deleteMod = async (modId: string) => {
    try {
      await invoke('uninstall_mod', {
        instanceId: instance.id,
        modId
      });
      await loadInstalledMods();
    } catch (err) {
      console.error('Failed to delete mod:', err);
      throw err;
    }
  };

  const deleteScreenshot = async (screenshotId: string) => {
    try {
      await invoke('delete_screenshot', {
        instanceId: instance.id,
        screenshotId
      });
      await loadScreenshots();
      if (selectedScreenshot?.id === screenshotId) {
        setSelectedScreenshot(null);
      }
    } catch (err) {
      console.error('Failed to delete screenshot:', err);
      throw err;
    }
  };

  const saveSettings = async (updatedInstance: Partial<MinecraftInstance>) => {
    try {
      await invoke('update_minecraft_instance', {
        instanceId: instance.id,
        updates: updatedInstance
      });
      return true;
    } catch (err) {
      console.error('Failed to save settings:', err);
      return false;
    }
  };

  return {
    activeTab,
    setActiveTab,
    searchQuery,
    setSearchQuery,
    searchResults,
    installedMods,
    loading,
    error,
    viewMode,
    setViewMode,
    hasMoreResults,
    isLoadingMore,
    jvmSettings,
    setJvmSettings,
    screenshots,
    selectedScreenshot,
    setSelectedScreenshot,
    screenshotLoading,
    installedResourcePacks,
    resourcePackLoading,
    searchMods,
    loadMoreMods,
    deleteMod,
    deleteScreenshot,
    saveSettings,
    loadInstalledMods,
    loadScreenshots,
    loadResourcePacks
  };
};