import { useState, useEffect, useCallback, useRef } from 'react';
import LauncherSidebar from './components/LauncherSidebar';
import HomeView from './components/HomeView';
import InstancesView from './components/InstancesView';
import SettingsView from './components/SettingsView';
import AccountsView from './components/AccountsView';
import CreateInstanceModal from './components/CreateInstanceModal';
import JavaInstallModal from './components/JavaInstallModal';
import { MinecraftInstance, MinecraftVersion, ModpackInfo, LauncherSettings, NewsItem, InstallProgressEvent, InstallCompleteEvent } from './types/minecraft';
import heroImage from './assets/hero.png';
import type { CSSProperties } from 'react';
import './index.css';

function App() {
  const [activeView, setActiveView] = useState('home');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showJavaInstallModal, setShowJavaInstallModal] = useState(false);
  const [pendingInstanceLaunch, setPendingInstanceLaunch] = useState<MinecraftInstance | null>(null);
  
  // Instance state
  const [instances, setInstances] = useState<MinecraftInstance[]>([]);
  const [launcherSettings, setLauncherSettings] = useState<LauncherSettings | null>(null);
  const [, setInstallProgress] = useState<Map<string, InstallProgressEvent>>(new Map());
  
  // Use ref to maintain current instances for event handlers
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

  // Detect all external launcher instances
  const detectExternalInstances = async (): Promise<MinecraftInstance[]> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const externalInstances = await invoke('detect_all_external_instances');
      return (externalInstances as any[]).map(instance => {
        const gameDir = instance.path || instance.game_dir || '';
        let status: 'ready' | 'invalid' = 'ready';
        let errorMessage: string | undefined;
        
        // Validate instance
        if (!gameDir) {
          status = 'invalid';
          errorMessage = 'Missing game directory path';
        } else if (!instance.name) {
          status = 'invalid';
          errorMessage = 'Missing instance name';
        } else if (!instance.version) {
          status = 'invalid';
          errorMessage = 'Missing Minecraft version';
        }
        
        return {
          id: instance.id,
          name: instance.name || 'Unknown Instance',
          version: instance.version || 'Unknown',
          gameDir,
          lastPlayed: instance.last_played ? new Date(instance.last_played) : undefined,
          totalPlayTime: instance.total_play_time || 0,
          isModded: instance.is_modded || false,
          modsCount: instance.mods_count || 0,
          isExternal: true,
          externalLauncher: instance.launcher_type as 'gdlauncher' | 'multimc' | 'prism' | 'modrinth',
          modpack: instance.modpack,
          modpackVersion: instance.modpack_version,
          icon: instance.icon,
          status,
          errorMessage,
        };
      });
    } catch (error) {
      console.error('Failed to detect external launcher instances:', error);
      return [];
    }
  };

  // Ensure Java is available
  const ensureJavaAvailable = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('get_bundled_java_path');
      console.log('Bundled Java is available');
    } catch (error) {
      console.log('Bundled Java not found, will download when needed');
    }
  };

  // Load instances and settings on startup
  useEffect(() => {
    const loadData = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Load stored instances
        const storedInstances = await invoke('load_instances') as MinecraftInstance[];
        console.log('Loaded stored instances from backend:', storedInstances);
        
        // Validate stored instances have gameDir and add status
        const validStoredInstances = storedInstances.filter(instance => {
          const gameDir = (instance as any).gameDir || (instance as any).game_dir;
          console.log('Validating stored instance:', {
            id: instance.id,
            name: instance.name,
            gameDir: gameDir,
            rawInstance: instance
          });
          
          if (!gameDir) {
            console.warn('Invalid stored instance found with missing gameDir:', instance.name || instance.id);
            return false;
          }
          return true;
        }).map(instance => {
          // Convert snake_case to camelCase for frontend compatibility
          const rawInstance = instance as any;
          return {
            id: rawInstance.id,
            name: rawInstance.name,
            version: rawInstance.version,
            modpack: rawInstance.modpack,
            modpackVersion: rawInstance.modpack_version,
            gameDir: rawInstance.game_dir || rawInstance.gameDir,
            javaPath: rawInstance.java_path,
            jvmArgs: rawInstance.jvm_args,
            lastPlayed: rawInstance.last_played ? new Date(rawInstance.last_played) : undefined,
            totalPlayTime: rawInstance.total_play_time || 0,
            icon: rawInstance.icon,
            isModded: rawInstance.is_modded || false,
            modsCount: rawInstance.mods_count || 0,
            isExternal: rawInstance.is_external,
            externalLauncher: rawInstance.external_launcher,
            status: rawInstance.status || 'ready' as const
          } as MinecraftInstance;
        });
        
        console.log('Valid stored instances after filtering:', validStoredInstances);
        
        // Load external instances
        const externalInstances = await detectExternalInstances();
        
        // Filter external instances to ensure gameDir exists
        const validExternalInstances = externalInstances.filter(instance => {
          if (!instance.gameDir) {
            console.warn('Invalid external instance found with missing gameDir:', instance.name || instance.id);
            return false;
          }
          return true;
        });
        
        // Combine both
        const allInstances = [...validStoredInstances, ...validExternalInstances];
        console.log('Final combined instances:', allInstances);
        setInstances(allInstances);
        
        // Load launcher settings
        const backendSettings = await invoke('get_launcher_settings') as LauncherSettings;
        setLauncherSettings(backendSettings);
        setSettings(backendSettings);
        
      } catch (error) {
        console.error('Failed to load launcher data:', error);
        // Fallback to external instances only
        const externalInstances = await detectExternalInstances();
        const validExternalInstances = externalInstances.filter(instance => {
          if (!instance.gameDir) {
            console.warn('Invalid external instance found with missing gameDir:', instance.name || instance.id);
            return false;
          }
          return true;
        });
        setInstances(validExternalInstances);
      }
    };

    // Set up event listeners
    const setupEventListeners = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
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

    loadData();
    setupEventListeners();
    ensureJavaAvailable();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const [settings, setSettings] = useState<LauncherSettings>({
    default_memory: 4096,
    default_jvm_args: ['-XX:+UnlockExperimentalVMOptions', '-XX:+UseG1GC'],
    instances_dir: '/minecraft/instances',
    downloads_dir: '/minecraft/downloads',
    theme: 'dark',
    auto_update: true,
    keepLauncherOpen: true,
    showSnapshots: false,
    javaPath: '',
    maxMemory: 4096,
    minMemory: 1024,
    jvmArgs: ['-XX:+UnlockExperimentalVMOptions', '-XX:+UseG1GC'],
    gameDir: '/minecraft',
  });

  const mockVersions: MinecraftVersion[] = [
    { id: '1.20.4', type: 'release', releaseTime: '2023-12-07T12:00:00Z', url: '' },
    { id: '1.20.3', type: 'release', releaseTime: '2023-12-05T12:00:00Z', url: '' },
    { id: '1.20.2', type: 'release', releaseTime: '2023-09-21T12:00:00Z', url: '' },
    { id: '24w06a', type: 'snapshot', releaseTime: '2024-02-07T12:00:00Z', url: '' },
  ];

  const mockModpacks: ModpackInfo[] = [
    {
      id: 'atm9',
      name: 'All The Mods 9',
      description: 'All the Mods started out as a private pack for just a few friends of mine that turned into something others wanted to play!',
      author: 'ATM Team',
      version: '0.2.59',
      minecraftVersion: '1.20.1',
      downloadUrl: '',
      iconUrl: '',
    },
    {
      id: 'create-ab',
      name: 'Create: Above and Beyond',
      description: 'A skyblock pack designed around the Create mod',
      author: 'simibubi',
      version: '1.4.0',
      minecraftVersion: '1.18.2',
      downloadUrl: '',
      iconUrl: '',
    },
  ];

  const mockNews: NewsItem[] = [
    {
      id: '1',
      title: 'Minecraft 1.21 - The Tricky Trials Update',
      summary: 'Brave new challenges with the Trial Chambers, fight the Breeze and Bogged, and collect their unique drops!',
      content: '',
      publishedAt: '2024-06-13T12:00:00Z',
      category: 'minecraft',
      imageUrl: '',
    },
    {
      id: '2',
      title: 'Launcher Update v2.0.0',
      summary: 'Major launcher overhaul with improved UI, better performance, and new features.',
      content: '',
      publishedAt: '2024-01-15T12:00:00Z',
      category: 'launcher',
    },
  ];

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const mockAccounts = [
    {
      id: '1',
      username: 'Player123',
      email: 'player@example.com',
      type: 'microsoft' as const,
      isActive: true,
      lastUsed: new Date(Date.now() - 86400000),
    },
    {
      id: '2',
      username: 'OfflinePlayer',
      email: '',
      type: 'offline' as const,
      isActive: false,
    },
  ];

  const handleCreateInstance = async (data: {
    name: string;
    version: string;
    modpack?: string;
    modpackVersion?: string;
  }) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
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
      await invoke('install_minecraft_version', {
        ...params,
        instanceId: instanceId
      });
      
      setShowCreateModal(false);
    } catch (error) {
      console.error('Failed to create instance:', error);
      console.error('Error details:', JSON.stringify(error, null, 2));
      
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
  };

  const handlePlayInstance = async (instance: MinecraftInstance) => {
    // Validate gameDir before launching
    if (!instance.gameDir) {
      console.error('Cannot launch instance: gameDir is undefined or empty for instance:', instance.name);
      alert('Error: This instance has an invalid game directory and cannot be launched.');
      return;
    }
    
    // Update last played time
    setInstances(prev => 
      prev.map(inst => 
        inst.id === instance.id 
          ? { ...inst, lastPlayed: new Date() }
          : inst
      )
    );
    
    try {
      console.log('Preparing to launch instance at:', instance.gameDir);
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Get bundled Java path, download if needed
      let javaPath: string;
      try {
        javaPath = await invoke('get_bundled_java_path') as string;
        console.log('Using bundled Java:', javaPath);
      } catch (javaError) {
        console.log('Bundled Java not found, showing install modal...');
        setPendingInstanceLaunch(instance);
        setShowJavaInstallModal(true);
        return;
      }
      
      await invoke('launch_instance', {
        instanceId: instance.id,
        instancePath: instance.gameDir,
        version: instance.version,
        javaPath,
        memory: settings.maxMemory || 4096,
        jvmArgs: settings.jvmArgs || []
      });
      console.log('Successfully launched instance:', instance.name);
    } catch (error) {
      console.error('Failed to launch instance:', error);
      alert(`Failed to launch ${instance.name}: ${error}`);
    }
  };

  const handleJavaInstallComplete = async (javaPath: string) => {
    setShowJavaInstallModal(false);
    
    if (pendingInstanceLaunch) {
      // Launch the pending instance with the new Java path
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('launch_instance', {
          instanceId: pendingInstanceLaunch.id,
          instancePath: pendingInstanceLaunch.gameDir,
          version: pendingInstanceLaunch.version,
          javaPath,
          memory: settings.maxMemory || 4096,
          jvmArgs: settings.jvmArgs || []
        });
        console.log('Successfully launched instance:', pendingInstanceLaunch.name);
      } catch (error) {
        console.error('Failed to launch instance after Java install:', error);
        alert(`Failed to launch ${pendingInstanceLaunch.name}: ${error}`);
      }
      
      setPendingInstanceLaunch(null);
    }
  };

  const handleJavaInstallCancel = () => {
    setShowJavaInstallModal(false);
    setPendingInstanceLaunch(null);
  };

  const handleEditInstance = (instance: MinecraftInstance) => {
    console.log('Editing instance:', instance.name);
  };

  const handleDeleteInstance = async (instance: MinecraftInstance) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      if (!instance.isExternal) {
        // Delete from storage
        await invoke('delete_instance', { instanceId: instance.id });
      }
      
      // Remove from UI
      setInstances(prev => prev.filter(inst => inst.id !== instance.id));
    } catch (error) {
      console.error('Failed to delete instance:', error);
    }
  };

  const handleUpdateSettings = async (newSettings: LauncherSettings) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_launcher_settings', { settings: newSettings });
      setSettings(newSettings);
      setLauncherSettings(newSettings);
    } catch (error) {
      console.error('Failed to update settings:', error);
    }
  };

  const handleOpenFolder = async (folderPath: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('open_folder', { path: folderPath });
    } catch (error) {
      console.error('Failed to open folder:', error);
    }
  };

  const handleOpenInstanceFolder = async (instance: MinecraftInstance) => {
    // Validate gameDir before opening
    if (!instance.gameDir) {
      console.error('Cannot open folder: gameDir is undefined or empty for instance:', instance.name);
      alert('Error: This instance has an invalid game directory.');
      return;
    }
    
    if (instance.isExternal) {
      await handleOpenFolder(instance.gameDir);
    } else {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('open_instance_folder', { instanceId: instance.id });
      } catch (error) {
        console.error('Failed to open instance folder:', error);
        // Fallback to opening the gameDir directly
        await handleOpenFolder(instance.gameDir);
      }
    }
  };

  const renderActiveView = () => {
    console.log('Rendering view with instances:', instances);
    switch (activeView) {
      case 'home':
        return (
          <HomeView
            recentInstances={instances.sort((a, b) => {
              if (!a.lastPlayed && !b.lastPlayed) return 0;
              if (!a.lastPlayed) return 1;
              if (!b.lastPlayed) return -1;
              return b.lastPlayed.getTime() - a.lastPlayed.getTime();
            })}
            news={mockNews}
            onCreateInstance={() => setShowCreateModal(true)}
            onPlayInstance={handlePlayInstance}
            onEditInstance={handleEditInstance}
            onDeleteInstance={handleDeleteInstance}
            onOpenFolder={handleOpenInstanceFolder}
          />
        );
      case 'instances':
        return (
          <InstancesView
            instances={instances}
            onCreateInstance={() => setShowCreateModal(true)}
            onPlayInstance={handlePlayInstance}
            onEditInstance={handleEditInstance}
            onDeleteInstance={handleDeleteInstance}
            onOpenFolder={handleOpenInstanceFolder}
          />
        );
      case 'settings':
        return (
          <SettingsView
            settings={settings}
            onUpdateSettings={handleUpdateSettings}
            onOpenFolder={handleOpenFolder}
          />
        );
      case 'accounts':
        return (
          <AccountsView
            onSetActiveAccount={(id) => console.log('Setting active account:', id)}
          />
        );
      default:
        return <div className="flex-1 p-6 text-white">View not implemented yet</div>;
    }
  };

  return (<div>
      <div className="h-screen bg-stone-950 flex relative overflow-hidden">
        {/* Hero Background Image */}
        <div className="absolute inset-0">
          <img
            src={heroImage}
            alt="Hero Background"
            className="w-full h-full object-cover blur-sm"
          />
          {/* Dark overlay for UI readability */}
          <div className="absolute inset-0 bg-black/60"></div>
        </div>
    
        {/* Main Content */}
        <div className="relative z-10 flex w-full">
          <LauncherSidebar
            activeView={activeView}
            onViewChange={setActiveView}
          />
    
            {/* Titlebar */}
            <div className="flex flex-col flex-1">
            <div
              className="bg-stone-900/60 backdrop-blur-sm border-r border-amber-600/30 h-9 flex items-center justify-between"
              style={{ WebkitAppRegion: 'drag' } as CSSProperties}
            >
              <div className="flex-1"></div>
              {/* Windows Buttons */}
              <div
                className="flex items-center gap-1 px-2"
                style={{ WebkitAppRegion: 'no-drag' } as CSSProperties}
              >
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-stone-800 rounded"
                title="Minimize"
                onClick={async (e) => {
                  e.stopPropagation();
                  const { getCurrentWindow } = await import('@tauri-apps/api/window');
                  const win = getCurrentWindow();
                  win.minimize();
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14">
                <rect x="3" y="10" width="8" height="1.5" fill="#eab308" />
                </svg>
              </button>
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-stone-800 rounded"
                title="Maximize"
                onClick={async (e) => {
                  e.stopPropagation();
                  const { getCurrentWindow } = await import('@tauri-apps/api/window');
                  const win = getCurrentWindow();
                  const isMaximized = await win.isMaximized();
                  if (isMaximized) {
                    win.unmaximize();
                  } else {
                    win.maximize();
                  }
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14">
                <rect x="3" y="3" width="8" height="8" stroke="#eab308" strokeWidth="1.5" fill="none" />
                </svg>
              </button>
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-red-700 rounded"
                title="Close"
                onClick={async (e) => {
                  e.stopPropagation();
                  const { getCurrentWindow } = await import('@tauri-apps/api/window');
                  const win = getCurrentWindow();
                  win.close();
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14">
                <line x1="4" y1="4" x2="10" y2="10" stroke="#fff" strokeWidth="1.5" />
                <line x1="10" y1="4" x2="4" y2="10" stroke="#fff" strokeWidth="1.5" />
                </svg>
              </button>
              </div>
            </div>
            {renderActiveView()}
            </div>
    
          <CreateInstanceModal
            isOpen={showCreateModal}
            onClose={() => setShowCreateModal(false)}
            onCreateInstance={handleCreateInstance}
            minecraftVersions={mockVersions}
            popularModpacks={mockModpacks}
          />
    
          <JavaInstallModal
            isOpen={showJavaInstallModal}
            onClose={handleJavaInstallCancel}
            onInstallComplete={handleJavaInstallComplete}
          />
        </div>
      </div>
  </div>
  );
}

export default App;