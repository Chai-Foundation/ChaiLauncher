import React, { useState } from 'react';
import { Package, Download, Settings } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { MinecraftVersion, ModpackInfo } from '../../../types/minecraft';
import { Modal, Button } from '../../ui';

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

  const renderStepIndicator = () => (
    <div className="flex items-center mb-6">
      {[1, 2, 3].map((stepNumber) => (
        <React.Fragment key={stepNumber}>
          <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
            stepNumber <= step 
              ? 'bg-secondary-600 text-white' 
              : 'bg-primary-700 text-primary-300'
          }`}>
            {stepNumber}
          </div>
          {stepNumber < 3 && (
            <div className={`h-px flex-1 mx-4 ${
              stepNumber < step ? 'bg-secondary-600' : 'bg-primary-700'
            }`} />
          )}
        </React.Fragment>
      ))}
    </div>
  );

  const renderStep1 = () => (
    <motion.div
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="space-y-4"
    >
      <h3 className="text-lg font-medium text-white mb-4">Choose Instance Type</h3>
      
      <div className="grid gap-4">
        <button
          onClick={() => setInstanceType('vanilla')}
          className={`p-4 rounded-lg border-2 text-left transition-all ${
            instanceType === 'vanilla'
              ? 'border-secondary-600 bg-secondary-600/10'
              : 'border-primary-600 hover:border-primary-500'
          }`}
        >
          <div className="flex items-center gap-3">
            <Package className="text-secondary-500" size={24} />
            <div>
              <h4 className="text-white font-medium">Vanilla Minecraft</h4>
              <p className="text-primary-400 text-sm">Pure Minecraft experience</p>
            </div>
          </div>
        </button>

        <button
          onClick={() => setInstanceType('modpack')}
          className={`p-4 rounded-lg border-2 text-left transition-all ${
            instanceType === 'modpack'
              ? 'border-secondary-600 bg-secondary-600/10'
              : 'border-primary-600 hover:border-primary-500'
          }`}
        >
          <div className="flex items-center gap-3">
            <Download className="text-secondary-500" size={24} />
            <div>
              <h4 className="text-white font-medium">Modpack</h4>
              <p className="text-primary-400 text-sm">Pre-configured modded experience</p>
            </div>
          </div>
        </button>

        <button
          onClick={() => setInstanceType('custom')}
          className={`p-4 rounded-lg border-2 text-left transition-all ${
            instanceType === 'custom'
              ? 'border-secondary-600 bg-secondary-600/10'
              : 'border-primary-600 hover:border-primary-500'
          }`}
        >
          <div className="flex items-center gap-3">
            <Settings className="text-secondary-500" size={24} />
            <div>
              <h4 className="text-white font-medium">Custom</h4>
              <p className="text-primary-400 text-sm">Build your own modded setup</p>
            </div>
          </div>
        </button>
      </div>
    </motion.div>
  );

  const renderStep2 = () => {
    if (instanceType === 'modpack') {
      return (
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -20 }}
          className="space-y-4"
        >
          <h3 className="text-lg font-medium text-white mb-4">Select Modpack</h3>
          <div className="grid gap-3 max-h-80 overflow-y-auto">
            {popularModpacks.map((modpack) => (
              <button
                key={modpack.id}
                onClick={() => {
                  setSelectedModpack(modpack);
                  setSelectedVersion(modpack.minecraftVersion);
                }}
                className={`p-4 rounded-lg border-2 text-left transition-all ${
                  selectedModpack?.id === modpack.id
                    ? 'border-secondary-600 bg-secondary-600/10'
                    : 'border-primary-600 hover:border-primary-500'
                }`}
              >
                <h4 className="text-white font-medium">{modpack.name}</h4>
                <p className="text-primary-400 text-sm mb-2">{modpack.description}</p>
                <div className="flex gap-4 text-xs text-primary-500">
                  <span>by {modpack.author}</span>
                  <span>v{modpack.version}</span>
                  <span>MC {modpack.minecraftVersion}</span>
                </div>
              </button>
            ))}
          </div>
        </motion.div>
      );
    }

    return (
      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: -20 }}
        className="space-y-4"
      >
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-medium text-white">Select Minecraft Version</h3>
          <label className="flex items-center gap-2 text-sm text-primary-300">
            <input
              type="checkbox"
              checked={showSnapshots}
              onChange={(e) => setShowSnapshots(e.target.checked)}
              className="rounded"
            />
            Include snapshots
          </label>
        </div>

        {versionsError && (
          <div className="p-3 bg-red-900/20 border border-red-700/50 rounded-lg text-red-400 text-sm">
            {versionsError}
          </div>
        )}

        <div className="max-h-64 overflow-y-auto border border-primary-700 rounded-lg">
          {versionsLoading ? (
            <div className="p-4 text-center text-primary-400">Loading versions...</div>
          ) : (
            <div className="divide-y divide-primary-700">
              {filteredVersions.map((version) => (
                <button
                  key={version.id}
                  onClick={() => setSelectedVersion(version.id)}
                  className={`w-full p-3 text-left hover:bg-primary-800 transition-colors ${
                    selectedVersion === version.id ? 'bg-secondary-900/30' : ''
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-white">{version.id}</span>
                    <span className={`px-2 py-1 rounded text-xs ${
                      version.type === 'release'
                        ? 'bg-green-900/30 text-green-400'
                        : 'bg-orange-900/30 text-orange-400'
                    }`}>
                      {version.type}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </motion.div>
    );
  };

  const renderStep3 = () => (
    <motion.div
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="space-y-4"
    >
      <h3 className="text-lg font-medium text-white mb-4">Name Your Instance</h3>
      <div>
        <label className="block text-sm text-primary-300 mb-2">Instance Name</label>
        <input
          type="text"
          value={instanceName}
          onChange={(e) => setInstanceName(e.target.value)}
          placeholder="My Minecraft Instance"
          className="w-full p-3 bg-primary-800 border border-primary-600 rounded-lg text-white placeholder-primary-500 focus:outline-none focus:border-secondary-600"
          autoFocus
        />
      </div>
      
      <div className="p-4 bg-primary-800/50 rounded-lg">
        <h4 className="text-white font-medium mb-2">Summary</h4>
        <div className="space-y-1 text-sm text-primary-300">
          <p><span className="text-primary-400">Type:</span> {instanceType}</p>
          <p><span className="text-primary-400">Version:</span> {selectedVersion}</p>
          {selectedModpack && (
            <p><span className="text-primary-400">Modpack:</span> {selectedModpack.name} v{selectedModpack.version}</p>
          )}
        </div>
      </div>
    </motion.div>
  );

  const renderStepContent = () => {
    switch (step) {
      case 1: return renderStep1();
      case 2: return renderStep2();
      case 3: return renderStep3();
      default: return null;
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Create New Instance" size="lg">
      <div className="p-6">
        {renderStepIndicator()}
        
        <div className="mb-6 min-h-[300px]">
          <AnimatePresence mode="wait">
            {renderStepContent()}
          </AnimatePresence>
        </div>

        <div className="flex justify-between">
          <Button
            variant="ghost"
            onClick={step === 1 ? onClose : handleBack}
          >
            {step === 1 ? 'Cancel' : 'Back'}
          </Button>

          <Button
            onClick={step === 3 ? handleCreate : handleNext}
            disabled={!canProceed()}
          >
            {step === 3 ? 'Create Instance' : 'Next'}
          </Button>
        </div>
      </div>
    </Modal>
  );
};

export default CreateInstanceModal;