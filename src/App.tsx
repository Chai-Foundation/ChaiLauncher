import { useState, useEffect, useCallback } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import LauncherSidebar from './components/LauncherSidebar';
import HomeView from './components/HomeView';
import InstancesView from './components/InstancesView';
import ServersView from './components/ServersView';
import ModpackBrowser from './components/ModpackBrowser';
import SettingsView from './components/SettingsView';
import AccountsView from './components/AccountsView';
import JavaInstallModal from './components/JavaInstallModal';
import InstanceSettingsModal from './components/InstanceSettingsModal';
import { CreateInstanceModal, ExitConfirmationModal } from './components/modals';
import { MinecraftInstance, ModpackInfo } from './types/minecraft';
import { useInstances, useSettings, useMinecraftVersions } from './hooks';
import { JavaService } from './services';
import heroImage from './assets/hero.png';
import type { CSSProperties } from 'react';
import './index.css';

function App() {
  const [activeView, setActiveView] = useState('home');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showJavaInstallModal, setShowJavaInstallModal] = useState(false);
  const [showExitConfirmationModal, setShowExitConfirmationModal] = useState(false);
  const [editingInstance, setEditingInstance] = useState<MinecraftInstance | null>(null);
  const [pendingInstanceLaunch, setPendingInstanceLaunch] = useState<MinecraftInstance | null>(null);
  const [requiredJavaVersion, setRequiredJavaVersion] = useState<number>(17);

  // Custom hooks
  const { settings, launcherSettings, updateSettings, openFolder } = useSettings();
  const { 
    instances, 
    instancesRef, 
    createInstance, 
    launchInstance, 
    deleteInstance, 
    openInstanceFolder,
    getInstallingInstances 
  } = useInstances(launcherSettings);
  const { minecraftVersions, versionsLoading, versionsError } = useMinecraftVersions();

  // Handle exit attempt
  const handleExitAttempt = useCallback(() => {
    const installing = getInstallingInstances();
    if (installing.length > 0) {
      setShowExitConfirmationModal(true);
      return false; // Prevent exit
    } else {
      return true; // Allow exit
    }
  }, [getInstallingInstances]);

  // Handle confirmed exit
  const handleConfirmedExit = useCallback(async () => {
    try {
      const currentWindow = getCurrentWindow();
      await currentWindow.close();
    } catch (error) {
      console.error('Error closing window:', error);
    }
  }, []);

  // Set up window close listeners
  useEffect(() => {
    const setupCloseListener = async () => {
      try {
        const currentWindow = getCurrentWindow();
        const unlistenCloseRequested = await currentWindow.onCloseRequested(async (event) => {
          // Check current instances using the ref to get latest state
          const installing = instancesRef.current.filter(instance => instance.status === 'installing');
          if (installing.length > 0) {
            // Prevent close and show modal
            event.preventDefault();
            setShowExitConfirmationModal(true);
          }
          // If no installations, allow the close to proceed normally
        });
        
        return unlistenCloseRequested;
      } catch (error) {
        console.error('Failed to set up close listener:', error);
      }
    };

    setupCloseListener();
  }, [instancesRef]);

  // Ensure Java is available on startup
  useEffect(() => {
    JavaService.ensureJavaAvailable();
  }, []);

  const handleCreateInstance = async (data: {
    name: string;
    version: string;
    modpack?: string;
    modpackVersion?: string;
  }) => {
    await createInstance(data);
    setShowCreateModal(false);
  };

  const handlePlayInstance = async (instance: MinecraftInstance) => {
    try {
      await launchInstance(instance, settings);
    } catch (error: any) {
      if (error.message === 'JAVA_REQUIRED') {
        try {
          const requiredVersion = await JavaService.getRequiredJavaVersion(instance.version);
          setRequiredJavaVersion(requiredVersion);
        } catch {
          setRequiredJavaVersion(17);
        }
        setPendingInstanceLaunch(instance);
        setShowJavaInstallModal(true);
      } else {
        console.error('Failed to launch instance:', error);
        alert(`Failed to launch ${instance.name}: ${error.message || error}`);
      }
    }
  };

  const handleJavaInstallComplete = async (javaPath: string) => {
    setShowJavaInstallModal(false);
    
    if (pendingInstanceLaunch) {
      try {
        await launchInstance(pendingInstanceLaunch, { ...settings, javaPath });
      } catch (error: any) {
        console.error('Failed to launch instance after Java install:', error);
        alert(`Failed to launch ${pendingInstanceLaunch.name}: ${error.message || error}`);
      }
      setPendingInstanceLaunch(null);
    }
  };

  const handleJavaInstallCancel = () => {
    setShowJavaInstallModal(false);
    setPendingInstanceLaunch(null);
  };

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
            onCreateInstance={() => setShowCreateModal(true)}
            onPlayInstance={handlePlayInstance}
            onEditInstance={setEditingInstance}
            onDeleteInstance={deleteInstance}
            onOpenFolder={openInstanceFolder}
          />
        );
      case 'instances':
        return (
          <InstancesView
            instances={instances}
            onCreateInstance={() => setShowCreateModal(true)}
            onPlayInstance={handlePlayInstance}
            onEditInstance={setEditingInstance}
            onDeleteInstance={deleteInstance}
            onOpenFolder={openInstanceFolder}
          />
        );
      case 'servers':
        return <ServersView instances={instances} />;
      case 'browse':
        return (
          <ModpackBrowser
            onCreateInstance={handleCreateInstance}
            launcherSettings={launcherSettings}
          />
        );
      case 'settings':
        return (
          <SettingsView
            settings={settings}
            onUpdateSettings={updateSettings}
            onOpenFolder={openFolder}
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

  // Context menu prevention
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.allow-context-menu')) {
        e.preventDefault();
      }
    };
    window.addEventListener('contextmenu', handleContextMenu);
    return () => {
      window.removeEventListener('contextmenu', handleContextMenu);
    };
  }, []);

  // Helper function to get proper background image src
  const getBackgroundImageSrc = () => {
    if (!launcherSettings?.background_image) {
      return heroImage;
    }

    const imagePath = launcherSettings.background_image.replace(/^["']|["']$/g, '');
    
    if (imagePath.startsWith('data:') || 
        imagePath.startsWith('http://') || 
        imagePath.startsWith('https://')) {
      return imagePath;
    }
    
    try {
      return convertFileSrc(imagePath);
    } catch (error) {
      console.warn('Failed to convert background image path:', error);
      return heroImage;
    }
  };

  return (
    <div className="min-h-screen h-full w-full bg-primary-950 flex flex-col">
      {/* Hero Background Image */}
      <div className="absolute inset-0">
        <img
          src={getBackgroundImageSrc()}
          alt="Hero Background"
          className="w-full h-full object-cover blur-sm"
          onError={(e) => {
            (e.target as HTMLImageElement).src = heroImage;
          }}
        />
        <div className="absolute inset-0 bg-black/60"></div>
      </div>

      {/* Main Content */}
      <div className="relative z-10 flex w-full">
        <LauncherSidebar
          activeView={activeView}
          onViewChange={setActiveView}
        />

        {/* Titlebar */}
        <div className="flex flex-col flex-1 min-h-0">
          <div
            className="bg-primary-900/60 backdrop-blur-sm border-r border-secondary-600/30 h-9 flex items-center justify-between"
            style={{ WebkitAppRegion: 'drag' } as CSSProperties}
          >
            <div className="flex-1"></div>
            {/* Windows Buttons */}
            <div
              className="flex items-center gap-1 px-2"
              style={{ WebkitAppRegion: 'no-drag' } as CSSProperties}
            >
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-primary-800 rounded"
                title="Minimize"
                onClick={async (e) => {
                  e.stopPropagation();
                  const { getCurrentWindow } = await import('@tauri-apps/api/window');
                  const win = getCurrentWindow();
                  win.minimize();
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14">
                  <rect x="3" y="10" width="8" height="1.5" fill="var(--secondary-500)" />
                </svg>
              </button>
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-primary-800 rounded"
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
                  <rect x="3" y="3" width="8" height="8" stroke="var(--secondary-500)" strokeWidth="1.5" fill="none" />
                </svg>
              </button>
              <button
                className="w-6 h-6 flex items-center justify-center hover:bg-red-700 rounded"
                title="Close"
                onClick={(e) => {
                  e.stopPropagation();
                  const shouldClose = handleExitAttempt();
                  if (shouldClose) {
                    handleConfirmedExit();
                  }
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14">
                  <line x1="4" y1="4" x2="10" y2="10" stroke="white" strokeWidth="1.5" />
                  <line x1="10" y1="4" x2="4" y2="10" stroke="white" strokeWidth="1.5" />
                </svg>
              </button>
            </div>
          </div>
          <main className="overflow-y-auto" style={{ height: 'calc(100vh - 40px)' }}>
            {renderActiveView()}
          </main>
        </div>

        {/* Modals */}
        <CreateInstanceModal
          isOpen={showCreateModal}
          onClose={() => setShowCreateModal(false)}
          onCreateInstance={handleCreateInstance}
          minecraftVersions={minecraftVersions}
          versionsLoading={versionsLoading}
          versionsError={versionsError}
          popularModpacks={mockModpacks}
        />

        <JavaInstallModal
          isOpen={showJavaInstallModal}
          onClose={handleJavaInstallCancel}
          onInstallComplete={handleJavaInstallComplete}
          requiredJavaVersion={requiredJavaVersion}
        />

        {editingInstance && (
          <InstanceSettingsModal
            isOpen={!!editingInstance}
            onClose={() => setEditingInstance(null)}
            instance={editingInstance}
          />
        )}

        <ExitConfirmationModal
          isOpen={showExitConfirmationModal}
          onClose={() => setShowExitConfirmationModal(false)}
          onConfirmExit={handleConfirmedExit}
          installingInstances={getInstallingInstances()}
        />
      </div>
    </div>
  );
}

export default App;