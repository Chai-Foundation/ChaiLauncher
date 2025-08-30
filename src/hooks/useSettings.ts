import { useState, useEffect } from 'react';
import { LauncherSettings } from '../types/minecraft';
import { SettingsService } from '../services';
import { applyColorScheme } from '../utils/colors';

export const useSettings = () => {
  const [settings, setSettings] = useState<LauncherSettings>(SettingsService.getDefaultSettings());
  const [launcherSettings, setLauncherSettings] = useState<LauncherSettings | null>(null);

  // Apply default color scheme immediately on mount
  useEffect(() => {
    applyColorScheme(settings);
  }, []);

  // Apply color scheme when settings change
  useEffect(() => {
    if (settings || launcherSettings) {
      const activeSettings = launcherSettings || settings;
      applyColorScheme(activeSettings);
    }
  }, [settings, launcherSettings]);

  // Load settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const backendSettings = await SettingsService.loadSettings();
        setLauncherSettings(backendSettings);
        setSettings(backendSettings);
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    };

    loadSettings();
  }, []);

  const updateSettings = async (newSettings: LauncherSettings) => {
    try {
      await SettingsService.updateSettings(newSettings);
      setSettings(newSettings);
      setLauncherSettings(newSettings);
    } catch (error) {
      console.error('Failed to update settings:', error);
      throw error;
    }
  };

  const openFolder = async (folderPath: string) => {
    try {
      await SettingsService.openFolder(folderPath);
    } catch (error) {
      console.error('Failed to open folder:', error);
      throw error;
    }
  };

  return {
    settings,
    launcherSettings,
    updateSettings,
    openFolder
  };
};

export default useSettings;