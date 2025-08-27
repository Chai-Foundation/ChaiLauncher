import React, { useState } from 'react';
import { X, Package, Download, Settings } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import * as Dialog from '@radix-ui/react-dialog';
import { MinecraftVersion, ModpackInfo } from '../types/minecraft';

interface CreateInstanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreateInstance: (data: {
    name: string;
    version: string;
    modpack?: string;
    modpackVersion?: string;
  }) => void;
  minecraftVersions: MinecraftVersion[];
  versionsLoading?: boolean;
  versionsError?: string | null;
  popularModpacks: ModpackInfo[];
}

type InstanceType = 'vanilla' | 'modpack' | 'custom';

const CreateInstanceModal: React.FC<CreateInstanceModalProps> = ({
  isOpen,
  onClose,
  onCreateInstance,
  minecraftVersions,
  versionsLoading = false,
  versionsError = null,
  popularModpacks,
}) => {
  const [step, setStep] = useState(1);
  const [instanceType, setInstanceType] = useState<InstanceType>('vanilla');
  const [instanceName, setInstanceName] = useState('');
  const [selectedVersion, setSelectedVersion] = useState('');
  const [selectedModpack, setSelectedModpack] = useState<ModpackInfo | null>(null);
  const [showSnapshots, setShowSnapshots] = useState(false);

  const handleNext = () => {
    if (step < 3) setStep(step + 1);
  };

  const handleBack = () => {
    if (step > 1) setStep(step - 1);
  };

  const handleCreate = () => {
    onCreateInstance({
      name: instanceName,
      version: selectedVersion,
      modpack: selectedModpack?.name,
      modpackVersion: selectedModpack?.version,
    });
    onClose();
    resetForm();
  };

  const resetForm = () => {
    setStep(1);
    setInstanceType('vanilla');
    setInstanceName('');
    setSelectedVersion('');
    setSelectedModpack(null);
    setShowSnapshots(false);
  };

  const filteredVersions = minecraftVersions.filter(version => 
    showSnapshots || version.type === 'release'
  );

  const canProceed = () => {
    switch (step) {
      case 1:
        return instanceType !== null;
      case 2:
        return instanceType === 'modpack' ? selectedModpack !== null : selectedVersion !== '';
      case 3:
        return instanceName.trim() !== '';
      default:
        return false;
    }
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black bg-opacity-50 z-50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-primary-900/90 rounded-lg border border-primary-700 w-full max-w-2xl max-h-[90vh] overflow-hidden z-50">
          <div className="flex items-center justify-between p-6 border-b border-primary-700">
            <h2 className="text-xl font-semibold text-white">Create New Instance</h2>
            <Dialog.Close className="text-primary-400 hover:text-white transition-colors">
              <X size={20} />
            </Dialog.Close>
          </div>

          <div className="p-6">
            <div className="flex items-center mb-6">
              {[1, 2, 3].map((stepNumber) => (
                <React.Fragment key={stepNumber}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                    step >= stepNumber 
                      ? 'bg-secondary-600 text-white' 
                      : 'bg-primary-700 text-primary-400'
                  }`}>
                    {stepNumber}
                  </div>
                  {stepNumber < 3 && (
                    <div className={`flex-1 h-1 mx-4 ${
                      step > stepNumber ? 'bg-secondary-600' : 'bg-primary-700'
                    }`} />
                  )}
                </React.Fragment>
              ))}
            </div>

            <AnimatePresence mode="wait">
              {step === 1 && (
                <motion.div
                  key="step1"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="space-y-4"
                >
                  <h3 className="text-lg font-semibold text-white mb-4">Choose Instance Type</h3>
                  
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <button
                      onClick={() => setInstanceType('vanilla')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        instanceType === 'vanilla'
                          ? 'border-secondary-600 bg-secondary-600 bg-opacity-20'
                          : 'border-primary-600 hover:border-primary-500'
                      }`}
                    >
                      <Package className="mx-auto mb-2 text-green-500" size={32} />
                      <h4 className="font-semibold text-white">Vanilla</h4>
                      <p className="text-sm text-primary-400">Pure Minecraft experience</p>
                    </button>

                    <button
                      onClick={() => setInstanceType('modpack')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        instanceType === 'modpack'
                          ? 'border-secondary-600 bg-secondary-600 bg-opacity-20'
                          : 'border-primary-600 hover:border-primary-500'
                      }`}
                    >
                      <Download className="mx-auto mb-2 text-purple-500" size={32} />
                      <h4 className="font-semibold text-white">Modpack</h4>
                      <p className="text-sm text-primary-400">Pre-configured mod collection</p>
                    </button>

                    <button
                      onClick={() => setInstanceType('custom')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        instanceType === 'custom'
                          ? 'border-secondary-600 bg-secondary-600 bg-opacity-20'
                          : 'border-primary-600 hover:border-primary-500'
                      }`}
                    >
                      <Settings className="mx-auto mb-2 text-orange-500" size={32} />
                      <h4 className="font-semibold text-white">Custom</h4>
                      <p className="text-sm text-primary-400">Build your own setup</p>
                    </button>
                  </div>
                </motion.div>
              )}

              {step === 2 && (
                <motion.div
                  key="step2"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="space-y-4"
                >
                  {instanceType === 'modpack' ? (
                    <>
                      <h3 className="text-lg font-semibold text-white mb-4">Select Modpack</h3>
                      <div className="grid grid-cols-1 gap-3 max-h-64 overflow-y-auto">
                        {popularModpacks.map((modpack) => (
                          <button
                            key={modpack.id}
                            onClick={() => setSelectedModpack(modpack)}
                            className={`p-4 rounded-lg border text-left transition-colors ${
                              selectedModpack?.id === modpack.id
                                ? 'border-secondary-600 bg-secondary-600 bg-opacity-20'
                                : 'border-primary-600 hover:border-primary-500'
                            }`}
                          >
                            <div className="flex items-center gap-3">
                              {modpack.iconUrl && (
                                <img src={modpack.iconUrl} alt={modpack.name} className="w-12 h-12 rounded" />
                              )}
                              <div className="flex-1">
                                <h4 className="font-semibold text-white">{modpack.name}</h4>
                                <p className="text-sm text-primary-400">{modpack.description}</p>
                                <p className="text-xs text-primary-500">
                                  Minecraft {modpack.minecraftVersion} • by {modpack.author}
                                </p>
                              </div>
                            </div>
                          </button>
                        ))}
                      </div>
                    </>
                  ) : (
                    <>
                      <h3 className="text-lg font-semibold text-white mb-4">Select Minecraft Version</h3>
                      
                      <div className="flex items-center gap-2 mb-4">
                        <input
                          type="checkbox"
                          id="showSnapshots"
                          checked={showSnapshots}
                          onChange={(e) => setShowSnapshots(e.target.checked)}
                          className="rounded border-primary-600 bg-primary-700"
                        />
                        <label htmlFor="showSnapshots" className="text-sm text-primary-300">
                          Show snapshots and beta versions
                        </label>
                      </div>

                      {/* Version loading state */}
                      {versionsLoading && (
                        <div className="flex items-center justify-center p-8">
                          <div className="flex items-center gap-3 text-primary-400">
                            <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-secondary-600"></div>
                            <span>Loading Minecraft versions...</span>
                          </div>
                        </div>
                      )}

                      {/* Version loading error */}
                      {versionsError && (
                        <div className="bg-red-900/20 border border-red-600/30 rounded-lg p-4 mb-4">
                          <div className="flex items-start gap-3">
                            <div className="flex-shrink-0 w-5 h-5 text-red-400 mt-0.5">
                              <svg fill="currentColor" viewBox="0 0 20 20">
                                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                              </svg>
                            </div>
                            <div className="flex-1">
                              <h4 className="text-red-300 font-medium mb-1">Version Loading Failed</h4>
                              <p className="text-red-200 text-sm leading-relaxed">{versionsError}</p>
                              <p className="text-red-300 text-sm mt-2">
                                Using fallback versions. Please check your internet connection or report this issue for debugging.
                              </p>
                            </div>
                          </div>
                        </div>
                      )}

                      {/* Version grid */}
                      {!versionsLoading && (
                        <div className="grid grid-cols-2 gap-2 max-h-64 overflow-y-auto">
                          {filteredVersions.map((version) => (
                            <button
                              key={version.id}
                              onClick={() => setSelectedVersion(version.id)}
                              className={`p-3 rounded-lg border text-left transition-colors ${
                                selectedVersion === version.id
                                  ? 'border-secondary-600 bg-secondary-600 bg-opacity-20'
                                  : 'border-primary-600 hover:border-primary-500'
                              }`}
                            >
                              <div className="font-semibold text-white">{version.id}</div>
                              <div className="text-xs text-primary-400 capitalize">{version.type}</div>
                            </button>
                          ))}
                        </div>
                      )}
                    </>
                  )}
                </motion.div>
              )}

              {step === 3 && (
                <motion.div
                  key="step3"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="space-y-4"
                >
                  <h3 className="text-lg font-semibold text-white mb-4">Instance Details</h3>
                  
                  <div>
                    <label className="block text-sm font-medium text-primary-300 mb-2">
                      Instance Name
                    </label>
                    <input
                      type="text"
                      value={instanceName}
                      onChange={(e) => setInstanceName(e.target.value)}
                      placeholder="My Awesome Instance"
                      className="w-full px-3 py-2 bg-primary-700 border border-primary-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-secondary-500"
                    />
                  </div>

                  <div className="bg-primary-700 rounded-lg p-4">
                    <h4 className="font-semibold text-white mb-2">Summary</h4>
                    <div className="space-y-1 text-sm text-primary-300">
                      <p><span className="text-primary-400">Type:</span> {instanceType}</p>
                      {instanceType === 'modpack' && selectedModpack && (
                        <>
                          <p><span className="text-primary-400">Modpack:</span> {selectedModpack.name}</p>
                          <p><span className="text-primary-400">Version:</span> {selectedModpack.version}</p>
                          <p><span className="text-primary-400">Minecraft:</span> {selectedModpack.minecraftVersion}</p>
                        </>
                      )}
                      {instanceType !== 'modpack' && (
                        <p><span className="text-primary-400">Minecraft Version:</span> {selectedVersion}</p>
                      )}
                    </div>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            <div className="flex justify-between mt-6">
              <button
                onClick={handleBack}
                disabled={step === 1}
                className="px-4 py-2 text-primary-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Back
              </button>
              
              <div className="space-x-3">
                <button
                  onClick={onClose}
                  className="px-4 py-2 text-primary-400 hover:text-white transition-colors"
                >
                  Cancel
                </button>
                {step < 3 ? (
                  <button
                    onClick={handleNext}
                    disabled={!canProceed()}
                    className="px-6 py-2 bg-secondary-600 hover:bg-secondary-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Next
                  </button>
                ) : (
                  <button
                    onClick={handleCreate}
                    disabled={!canProceed()}
                    className="px-6 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Create Instance
                  </button>
                )}
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default CreateInstanceModal;