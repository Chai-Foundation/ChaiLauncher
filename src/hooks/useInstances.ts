import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { MinecraftInstance, InstallProgressEvent, InstallCompleteEvent, LauncherSettings } from '../types/minecraft';
import { InstanceService, JavaService } from '../services';

export const useInstances = (launcherSettings: LauncherSettings | null) => {
  const [instances, setInstances] = useState<MinecraftInstance[]>([]);
  const [installProgress, setInstallProgress] = useState<Map<string, InstallProgressEvent>>(new Map());
  const instancesRef = useRef<MinecraftInstance[]>([]);

  // Update ref whenever instances change
  useEffect(() => {
    instancesRef.current = instances;
  }, [instances]);

  // Stable progress update handler with throttling
  const handleProgressUpdate = useCallback((event: any) => {
    const instanceId = event.instance_id || event.instanceId;
    const newProgress = Math.round(event.progress);
    
    setInstances(prev => {
      const updated = prev.map(inst => {
        if (inst.id === instanceId) {
          // Only update if progress changed by at least 1% to reduce flickering
          const currentProgress = Math.round(inst.installProgress || 0);
          if (Math.abs(newProgress - currentProgress) >= 1 || inst.status !== 'installing') {
            const updatedInstance = { 
              ...inst, 
              installProgress: newProgress, 
              status: 'installing' as const,
            };
            console.log(`Updating instance ${inst.name}: progress=${newProgress}%`);
            return updatedInstance;
          }
        }
        return inst;
      });
      return updated;
    });
  }, []);

  // Stable completion handler
  const handleInstallComplete = useCallback(async (event: any) => {
    console.log('Install complete event received:', event);
    const instanceId = event.instance_id || event.instanceId;
    setInstallProgress(prev => {
      const newMap = new Map(prev);
      newMap.delete(instanceId);
      return newMap;
    });
    
    if (event.success) {
      setInstances(prev => 
        prev.map(inst => 
          inst.id === instanceId
            ? { ...inst, status: 'ready', installProgress: 100 }
            : inst
        )
      );
    } else {
      setInstances(prev => 
        prev.map(inst => 
          inst.id === instanceId
            ? { ...inst, status: 'failed', errorMessage: event.error || 'Installation failed' }
            : inst
        )
      );
    }
  }, []);

  // Load instances on mount
  useEffect(() => {
    const loadInstances = async () => {
      try {
        // Load stored instances
        const storedInstances = await InstanceService.loadInstances();
        console.log('Loaded stored instances from backend:', storedInstances);
        
        // Load external instances
        const externalInstances = await InstanceService.detectExternalInstances();
        
        // Combine both
        const allInstances = [...storedInstances, ...externalInstances];
        console.log('Final combined instances:', allInstances);
        setInstances(allInstances);
        
      } catch (error) {
        console.error('Failed to load launcher data:', error);
        // Fallback to external instances only
        const externalInstances = await InstanceService.detectExternalInstances();
        setInstances(externalInstances);
      }
    };

    // Set up event listeners
    const setupEventListeners = async () => {
      try {
        // Listen for install progress
        const unlistenProgress = await listen<InstallProgressEvent>('install_progress', (event) => {
          handleProgressUpdate(event.payload);
        });
        
        // Listen for install completion
        const unlistenComplete = await listen<InstallCompleteEvent>('install_complete', (event) => {
          handleInstallComplete(event.payload);
        });

        return () => {
          unlistenProgress();
          unlistenComplete();
        };
      } catch (error) {
        console.error('Failed to set up event listeners:', error);
      }
    };

    loadInstances();
    setupEventListeners();
  }, [handleProgressUpdate, handleInstallComplete]);

  // Automatic orphaned instance scanning
  useEffect(() => {
    const importOrphanedInstances = async () => {
      try {
        const imported = await InstanceService.importOrphanedInstances();
        
        if (imported.length > 0) {
          console.log('🔄 Auto-imported orphaned instances:', imported);
          
          // Reload instances to show the newly imported ones
          const updatedInstances = await InstanceService.loadInstances();
          setInstances(updatedInstances);
        }
      } catch (error) {
        console.log('Orphaned instance scan failed (normal if no orphans):', error);
      }
    };

    // Run immediately on mount
    importOrphanedInstances();
  }, []);

  const createInstance = useCallback(async (data: {
    name: string;
    version: string;
    modpack?: string;
    modpackVersion?: string;
  }) => {
    try {
      const params = {
        versionId: data.version,
        instanceName: data.name,
        gameDir: launcherSettings?.instances_dir || '/minecraft',
      };
      
      // Generate instance ID upfront to avoid timing issues
      const instanceId = `instance-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
      
      // Create instance with real ID and installing status BEFORE starting installation
      const newInstance: MinecraftInstance = {
        id: instanceId,
        name: data.name,
        version: data.version,
        gameDir: `${launcherSettings?.instances_dir || '/minecraft'}/${data.name}`,
        totalPlayTime: 0,
        isModded: !!data.modpack,
        modsCount: 0,
        status: 'installing',
        installProgress: 0,
        modpack: data.modpack,
        modpackVersion: data.modpackVersion,
      };
      
      // Add to instances BEFORE starting installation
      setInstances(prev => [...prev, newInstance]);
      
      // Start the installation with predefined instance ID
      await InstanceService.createInstance({
        ...params,
        instanceId: instanceId
      });
      
    } catch (error) {
      console.error('Failed to create instance:', error);
      
      // Mark any temp instances as failed
      setInstances(prev => 
        prev.map(inst => 
          inst.status === 'installing' 
            ? { 
                ...inst, 
                status: 'failed', 
                errorMessage: typeof error === 'string' 
                  ? error 
                  : error instanceof Error 
                    ? error.message 
                    : JSON.stringify(error) 
              }
            : inst
        )
      );
    }
  }, [launcherSettings]);

  const launchInstance = useCallback(async (instance: MinecraftInstance, settings: LauncherSettings) => {
    // Validate gameDir before launching
    if (!instance.gameDir) {
      console.error('Cannot launch instance: gameDir is undefined or empty for instance:', instance.name);
      throw new Error('This instance has an invalid game directory and cannot be launched.');
    }
    
    // Update last played time
    setInstances(prev => 
      prev.map(inst => 
        inst.id === instance.id 
          ? { ...inst, lastPlayed: new Date() }
          : inst
      )
    );
    
    console.log('Preparing to launch instance at:', instance.gameDir);
    
    // Get Java path
    const javaInfo = await JavaService.getJavaForInstance(instance.version);
    if (javaInfo.requiresInstall) {
      throw new Error('JAVA_REQUIRED');
    }
    
    await InstanceService.launchInstance({
      instanceId: instance.id,
      instancePath: instance.gameDir,
      version: instance.version,
      javaPath: javaInfo.javaPath,
      memory: settings.maxMemory || 4096,
      jvmArgs: settings.jvmArgs || []
    });
    
    console.log('Successfully launched instance:', instance.name);
  }, []);

  const deleteInstance = useCallback(async (instance: MinecraftInstance) => {
    try {
      if (!instance.isExternal) {
        await InstanceService.deleteInstance(instance.id);
      }
      
      // Remove from UI
      setInstances(prev => prev.filter(inst => inst.id !== instance.id));
    } catch (error) {
      console.error('Failed to delete instance:', error);
    }
  }, []);

  const openInstanceFolder = useCallback(async (instance: MinecraftInstance) => {
    if (!instance.gameDir) {
      console.error('Cannot open folder: gameDir is undefined or empty for instance:', instance.name);
      throw new Error('This instance has an invalid game directory.');
    }
    
    if (instance.isExternal) {
      const { SettingsService } = await import('../services');
      await SettingsService.openFolder(instance.gameDir);
    } else {
      await InstanceService.openInstanceFolder(instance.id);
    }
  }, []);

  const getInstallingInstances = useCallback(() => {
    return instances.filter(instance => instance.status === 'installing')
      .map(instance => ({
        name: instance.name,
        installProgress: instance.installProgress || 0
      }));
  }, [instances]);

  return {
    instances,
    instancesRef,
    createInstance,
    launchInstance,
    deleteInstance,
    openInstanceFolder,
    getInstallingInstances
  };
};

export default useInstances;