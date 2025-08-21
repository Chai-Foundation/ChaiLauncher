import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Download, CheckCircle } from 'lucide-react';

interface JavaInstallEvent {
  stage: string;
  progress: number;
}

interface JavaInstallModalProps {
  isOpen: boolean;
  onClose: () => void;
  onInstallComplete: (javaPath: string) => void;
}

const JavaInstallModal: React.FC<JavaInstallModalProps> = ({ isOpen, onClose, onInstallComplete }) => {
  const [installProgress, setInstallProgress] = useState<JavaInstallEvent>({
    stage: 'Preparing...',
    progress: 0
  });
  const [isInstalling, setIsInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [isComplete, setIsComplete] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      // Reset state when modal closes
      setInstallProgress({ stage: 'Preparing...', progress: 0 });
      setIsInstalling(false);
      setInstallError(null);
      setIsComplete(false);
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;

    const setupEventListener = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      
      const unlisten = await listen<JavaInstallEvent>('java_install_progress', (event) => {
        console.log('Java install progress:', event.payload);
        setInstallProgress(event.payload);
        
        if (event.payload.progress >= 100) {
          setIsComplete(true);
          setIsInstalling(false);
        }
      });

      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupEventListener().then(fn => { unlisten = fn; });

    return () => {
      if (unlisten) unlisten();
    };
  }, [isOpen]);

  const handleInstall = async () => {
    setIsInstalling(true);
    setInstallError(null);
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const javaPath = await invoke('download_and_install_java') as string;
      console.log('Java installation complete:', javaPath);
      onInstallComplete(javaPath);
    } catch (error: unknown) {
      console.error('Java installation failed:', error);
      setInstallError(String(error));
      setIsInstalling(false);
    }
  };

  const handleClose = () => {
    if (!isInstalling) {
      onClose();
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          onClick={handleClose}
        >
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            className="bg-stone-900 rounded-xl border border-amber-600/30 p-6 max-w-md w-full mx-4"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="text-center">
              <div className="mb-4">
                {isComplete ? (
                  <CheckCircle size={48} className="text-green-400 mx-auto mb-2" />
                ) : (
                  <Download size={48} className="text-amber-400 mx-auto mb-2" />
                )}
                <h2 className="text-xl font-bold text-white mb-2">
                  {isComplete ? 'Java Installed!' : 'Java Runtime Required'}
                </h2>
                <p className="text-stone-300 text-sm">
                  {isComplete 
                    ? 'Java has been successfully installed and is ready to use.'
                    : 'ChaiLauncher needs to download and install Java 17 to run Minecraft instances.'
                  }
                </p>
              </div>

              {isInstalling && (
                <div className="mb-4">
                  <div className="mb-2">
                    <div className="bg-stone-700 rounded-full h-3 overflow-hidden">
                      <motion.div
                        className="bg-gradient-to-r from-amber-500 to-amber-400 h-3 rounded-full"
                        initial={{ width: '0%' }}
                        animate={{ width: `${installProgress.progress}%` }}
                        transition={{ duration: 0.3 }}
                      />
                    </div>
                  </div>
                  <p className="text-stone-300 text-sm">
                    {installProgress.stage} ({Math.round(installProgress.progress)}%)
                  </p>
                </div>
              )}

              {installError && (
                <div className="mb-4 p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
                  <p className="text-red-400 text-sm">{installError}</p>
                </div>
              )}

              <div className="flex gap-3 justify-center">
                {!isInstalling && !isComplete && (
                  <>
                    <button
                      onClick={handleClose}
                      className="px-4 py-2 bg-stone-700 hover:bg-stone-600 text-white rounded-lg transition-colors"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleInstall}
                      className="px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg transition-colors flex items-center gap-2"
                    >
                      <Download size={16} />
                      Install Java
                    </button>
                  </>
                )}
                
                {isInstalling && (
                  <button
                    disabled
                    className="px-4 py-2 bg-stone-700 text-stone-400 rounded-lg cursor-not-allowed"
                  >
                    Installing...
                  </button>
                )}

                {isComplete && (
                  <button
                    onClick={handleClose}
                    className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg transition-colors flex items-center gap-2"
                  >
                    <CheckCircle size={16} />
                    Continue
                  </button>
                )}
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default JavaInstallModal;