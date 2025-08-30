import React, { useEffect } from 'react';
import { X, Server } from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';
import { DockerConnection, ServerDeploymentRequest } from '../types/servers';
import { useServerDeployment } from '../hooks/useServerDeployment';
import { BasicServerSettings } from './servers/BasicServerSettings';
import { AdvancedServerSettings } from './servers/AdvancedServerSettings';

interface DeployServerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onDeploy: (request: ServerDeploymentRequest) => void;
  instances: MinecraftInstance[];
  dockerConnections: DockerConnection[];
  selectedInstance?: MinecraftInstance | null;
}

const DeployServerModal: React.FC<DeployServerModalProps> = ({ 
  isOpen, 
  onClose, 
  onDeploy, 
  instances, 
  dockerConnections,
  selectedInstance 
}) => {
  const {
    formData,
    setFormData,
    envVarInput,
    setEnvVarInput,
    showAdvanced,
    setShowAdvanced,
    selectedInstanceData,
    resetForm,
    addEnvironmentVariable,
    removeEnvironmentVariable,
    validateForm,
    createDeploymentRequest
  } = useServerDeployment(instances, dockerConnections, selectedInstance);

  useEffect(() => {
    if (!isOpen) {
      resetForm();
    }
  }, [isOpen, resetForm]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const validationError = validateForm();
    if (validationError) {
      alert(validationError);
      return;
    }

    const request = createDeploymentRequest();
    onDeploy(request);
  };


  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-primary-800 rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-secondary-600/20 rounded-lg flex items-center justify-center">
              <Server className="w-5 h-5 text-secondary-400" />
            </div>
            <h2 className="text-xl font-bold text-white">Deploy Minecraft Server</h2>
          </div>
          <button
            onClick={onClose}
            className="text-primary-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <BasicServerSettings
            formData={formData}
            setFormData={setFormData}
            instances={instances}
            dockerConnections={dockerConnections}
            selectedInstanceData={selectedInstanceData}
            selectedInstance={selectedInstance}
          />

          <AdvancedServerSettings
            showAdvanced={showAdvanced}
            setShowAdvanced={setShowAdvanced}
            environmentVars={formData.environment_vars}
            envVarInput={envVarInput}
            setEnvVarInput={setEnvVarInput}
            onAddEnvVar={addEnvironmentVariable}
            onRemoveEnvVar={removeEnvironmentVariable}
          />

          <div className="flex justify-end gap-3 pt-6 border-t border-primary-600">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-primary-300 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={dockerConnections.filter(c => c.is_connected).length === 0 || instances.length === 0}
              className="px-6 py-2 bg-secondary-600 hover:bg-secondary-700 disabled:bg-secondary-600/50 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
            >
              Deploy Server
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default DeployServerModal;