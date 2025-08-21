import { useState, useEffect } from 'react';
import LauncherSidebar from './components/LauncherSidebar';
import HomeView from './components/HomeView';
import InstancesView from './components/InstancesView';
import SettingsView from './components/SettingsView';
import AccountsView from './components/AccountsView';
import CreateInstanceModal from './components/CreateInstanceModal';
import { MinecraftInstance, MinecraftVersion, ModpackInfo, LauncherSettings, NewsItem, InstallProgressEvent, InstallCompleteEvent } from './types/minecraft';
import heroImage from './assets/hero.png';
import './index.css';

function App() {
  const [activeView, setActiveView] = useState('home');
  const [showCreateModal, setShowCreateModal] = useState(false);
  
  // Instance state
  const [instances, setInstances] = useState<MinecraftInstance[]>([]);
  const [launcherSettings, setLauncherSettings] = useState<LauncherSettings | null>(null);
  const [installProgress, setInstallProgress] = useState<Map<string, InstallProgressEvent>>(new Map());

  // Detect all external launcher instances
  const detectExternalInstances = async (): Promise<MinecraftInstance[]> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const externalInstances = await invoke('detect_all_external_instances');
      return (externalInstances as any[]).map(instance => ({
        id: instance.id,
        name: instance.name,
        version: instance.version || 'Unknown',
        gameDir: instance.path,
        lastPlayed: instance.last_played ? new Date(instance.last_played) : undefined,
        totalPlayTime: instance.total_play_time || 0,
        isModded: instance.is_modded || false,
        modsCount: instance.mods_count || 0,
        isExternal: true,
        externalLauncher: instance.launcher_type as 'gdlauncher' | 'multimc' | 'prism' | 'modrinth',
        modpack: instance.modpack,
        modpackVersion: instance.modpack_version,
        icon: instance.icon,
      }));
    } catch (error) {
      console.error('Failed to detect external launcher instances:', error);
      return [];
    }
  };

  // Load instances and settings on startup
  useEffect(() => {
    const loadData = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Load stored instances
        const storedInstances = await invoke('load_instances') as MinecraftInstance[];
        
        // Load external instances
        const externalInstances = await detectExternalInstances();
        
        // Combine both
        setInstances([...storedInstances, ...externalInstances]);
        
        // Load launcher settings
        const settings = await invoke('get_launcher_settings') as LauncherSettings;
        setLauncherSettings(settings);
        
      } catch (error) {
        console.error('Failed to load launcher data:', error);
        // Fallback to external instances only
        const externalInstances = await detectExternalInstances();
        setInstances(externalInstances);
      }
    };

    // Set up event listeners
    const setupEventListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      
      // Listen for install progress
      const unlistenProgress = await listen<InstallProgressEvent>('install_progress', (event) => {
        setInstallProgress(prev => new Map(prev.set(event.payload.instanceId, event.payload)));
      });
      
      // Listen for install completion
      const unlistenComplete = await listen<InstallCompleteEvent>('install_complete', async (event) => {
        setInstallProgress(prev => {
          const newMap = new Map(prev);
          newMap.delete(event.payload.instanceId);
          return newMap;
        });
        
        if (event.payload.success) {
          // Reload instances
          try {
            const { invoke } = await import('@tauri-apps/api/core');
            const storedInstances = await invoke('load_instances') as MinecraftInstance[];
            const externalInstances = await detectExternalInstances();
            setInstances([...storedInstances, ...externalInstances]);
          } catch (error) {
            console.error('Failed to reload instances:', error);
          }
        }
      });
      
      return () => {
        unlistenProgress();
        unlistenComplete();
      };
    };

    loadData();
    setupEventListeners();
  }, []);

  const [settings, setSettings] = useState<LauncherSettings>({
    javaPath: '',
    maxMemory: 4096,
    minMemory: 1024,
    jvmArgs: ['-XX:+UnlockExperimentalVMOptions', '-XX:+UseG1GC'],
    gameDir: '/minecraft',
    keepLauncherOpen: true,
    showSnapshots: false,
    theme: 'dark',
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
      
      // Use the new installation system
      const instanceId = await invoke('install_minecraft_version', {
        versionId: data.version,
        instanceName: data.name,
        gameDir: launcherSettings?.instancesDir || '/minecraft',
      }) as string;
      
      console.log('Started installation for instance:', instanceId);
      setShowCreateModal(false);
    } catch (error) {
      console.error('Failed to create instance:', error);
    }
  };

  const handlePlayInstance = async (instance: MinecraftInstance) => {
    // Update last played time
    setInstances(prev => 
      prev.map(inst => 
        inst.id === instance.id 
          ? { ...inst, lastPlayed: new Date() }
          : inst
      )
    );
    
    try {
      // Launch Minecraft instance via Tauri
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('launch_instance', { 
        instanceId: instance.id,
        instancePath: instance.gameDir,
        version: instance.version,
        javaPath: settings.javaPath || 'java',
        memory: settings.maxMemory,
        jvmArgs: settings.jvmArgs
      });
      console.log('Launching instance:', instance.name);
    } catch (error) {
      console.error('Failed to launch instance:', error);
    }
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

  const renderActiveView = () => {
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
          />
        );
      case 'settings':
        return (
          <SettingsView
            settings={settings}
            onUpdateSettings={setSettings}
          />
        );
      case 'accounts':
        return (
          <AccountsView
            accounts={mockAccounts}
            onAddAccount={(type) => console.log('Adding account:', type)}
            onRemoveAccount={(id) => console.log('Removing account:', id)}
            onSetActiveAccount={(id) => console.log('Setting active account:', id)}
          />
        );
      default:
        return <div className="flex-1 p-6 text-white">View not implemented yet</div>;
    }
  };

  return (
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
        
        {renderActiveView()}
        
        <CreateInstanceModal
          isOpen={showCreateModal}
          onClose={() => setShowCreateModal(false)}
          onCreateInstance={handleCreateInstance}
          minecraftVersions={mockVersions}
          popularModpacks={mockModpacks}
        />
      </div>
    </div>
  );
}

export default App;