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
  popularModpacks: ModpackInfo[];
}

type InstanceType = 'vanilla' | 'modpack' | 'custom';

const CreateInstanceModal: React.FC<CreateInstanceModalProps> = ({
  isOpen,
  onClose,
  onCreateInstance,
  minecraftVersions,
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
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-stone-900/90 rounded-lg border border-stone-700 w-full max-w-2xl max-h-[90vh] overflow-hidden z-50">
          <div className="flex items-center justify-between p-6 border-b border-stone-700">
            <h2 className="text-xl font-semibold text-white">Create New Instance</h2>
            <Dialog.Close className="text-stone-400 hover:text-white transition-colors">
              <X size={20} />
            </Dialog.Close>
          </div>

          <div className="p-6">
            <div className="flex items-center mb-6">
              {[1, 2, 3].map((stepNumber) => (
                <React.Fragment key={stepNumber}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                    step >= stepNumber 
                      ? 'bg-amber-600 text-white' 
                      : 'bg-stone-700 text-stone-400'
                  }`}>
                    {stepNumber}
                  </div>
                  {stepNumber < 3 && (
                    <div className={`flex-1 h-1 mx-4 ${
                      step > stepNumber ? 'bg-amber-600' : 'bg-stone-700'
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
                          ? 'border-amber-600 bg-amber-600 bg-opacity-20'
                          : 'border-stone-600 hover:border-stone-500'
                      }`}
                    >
                      <Package className="mx-auto mb-2 text-green-500" size={32} />
                      <h4 className="font-semibold text-white">Vanilla</h4>
                      <p className="text-sm text-stone-400">Pure Minecraft experience</p>
                    </button>

                    <button
                      onClick={() => setInstanceType('modpack')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        instanceType === 'modpack'
                          ? 'border-amber-600 bg-amber-600 bg-opacity-20'
                          : 'border-stone-600 hover:border-stone-500'
                      }`}
                    >
                      <Download className="mx-auto mb-2 text-purple-500" size={32} />
                      <h4 className="font-semibold text-white">Modpack</h4>
                      <p className="text-sm text-stone-400">Pre-configured mod collection</p>
                    </button>

                    <button
                      onClick={() => setInstanceType('custom')}
                      className={`p-4 rounded-lg border-2 transition-colors ${
                        instanceType === 'custom'
                          ? 'border-amber-600 bg-amber-600 bg-opacity-20'
                          : 'border-stone-600 hover:border-stone-500'
                      }`}
                    >
                      <Settings className="mx-auto mb-2 text-orange-500" size={32} />
                      <h4 className="font-semibold text-white">Custom</h4>
                      <p className="text-sm text-stone-400">Build your own setup</p>
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
                                ? 'border-amber-600 bg-amber-600 bg-opacity-20'
                                : 'border-stone-600 hover:border-stone-500'
                            }`}
                          >
                            <div className="flex items-center gap-3">
                              {modpack.iconUrl && (
                                <img src={modpack.iconUrl} alt={modpack.name} className="w-12 h-12 rounded" />
                              )}
                              <div className="flex-1">
                                <h4 className="font-semibold text-white">{modpack.name}</h4>
                                <p className="text-sm text-stone-400">{modpack.description}</p>
                                <p className="text-xs text-stone-500">
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
                          className="rounded border-stone-600 bg-stone-700"
                        />
                        <label htmlFor="showSnapshots" className="text-sm text-stone-300">
                          Show snapshots and beta versions
                        </label>
                      </div>

                      <div className="grid grid-cols-2 gap-2 max-h-64 overflow-y-auto">
                        {filteredVersions.map((version) => (
                          <button
                            key={version.id}
                            onClick={() => setSelectedVersion(version.id)}
                            className={`p-3 rounded-lg border text-left transition-colors ${
                              selectedVersion === version.id
                                ? 'border-amber-600 bg-amber-600 bg-opacity-20'
                                : 'border-stone-600 hover:border-stone-500'
                            }`}
                          >
                            <div className="font-semibold text-white">{version.id}</div>
                            <div className="text-xs text-stone-400 capitalize">{version.type}</div>
                          </button>
                        ))}
                      </div>
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
                    <label className="block text-sm font-medium text-stone-300 mb-2">
                      Instance Name
                    </label>
                    <input
                      type="text"
                      value={instanceName}
                      onChange={(e) => setInstanceName(e.target.value)}
                      placeholder="My Awesome Instance"
                      className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white placeholder-stone-400 focus:outline-none focus:ring-2 focus:ring-amber-500"
                    />
                  </div>

                  <div className="bg-stone-700 rounded-lg p-4">
                    <h4 className="font-semibold text-white mb-2">Summary</h4>
                    <div className="space-y-1 text-sm text-stone-300">
                      <p><span className="text-stone-400">Type:</span> {instanceType}</p>
                      {instanceType === 'modpack' && selectedModpack && (
                        <>
                          <p><span className="text-stone-400">Modpack:</span> {selectedModpack.name}</p>
                          <p><span className="text-stone-400">Version:</span> {selectedModpack.version}</p>
                          <p><span className="text-stone-400">Minecraft:</span> {selectedModpack.minecraftVersion}</p>
                        </>
                      )}
                      {instanceType !== 'modpack' && (
                        <p><span className="text-stone-400">Minecraft Version:</span> {selectedVersion}</p>
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
                className="px-4 py-2 text-stone-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Back
              </button>
              
              <div className="space-x-3">
                <button
                  onClick={onClose}
                  className="px-4 py-2 text-stone-400 hover:text-white transition-colors"
                >
                  Cancel
                </button>
                {step < 3 ? (
                  <button
                    onClick={handleNext}
                    disabled={!canProceed()}
                    className="px-6 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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