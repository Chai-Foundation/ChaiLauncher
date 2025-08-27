import React, { useState, useEffect } from 'react';
import { X, Server, Settings } from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';
import { DockerConnection, ServerDeploymentRequest } from '../types/servers';

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
  const [formData, setFormData] = useState({
    name: '',
    minecraft_instance_id: selectedInstance?.id || '',
    docker_connection_id: '',
    port: 25565,
    max_players: 20,
    memory_limit: 2048,
    auto_start: true,
    environment_vars: {} as Record<string, string>
  });

  const [envVarInput, setEnvVarInput] = useState({ key: '', value: '' });
  const [showAdvanced, setShowAdvanced] = useState(false);

  useEffect(() => {
    if (selectedInstance) {
      setFormData(prev => ({
        ...prev,
        minecraft_instance_id: selectedInstance.id,
        name: prev.name || `${selectedInstance.name}-server`
      }));
    }
  }, [selectedInstance]);

  useEffect(() => {
    if (!isOpen) {
      // Reset form when modal closes
      setFormData({
        name: '',
        minecraft_instance_id: selectedInstance?.id || '',
        docker_connection_id: '',
        port: 25565,
        max_players: 20,
        memory_limit: 2048,
        auto_start: true,
        environment_vars: {}
      });
      setEnvVarInput({ key: '', value: '' });
      setShowAdvanced(false);
    }
  }, [isOpen, selectedInstance?.id]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      alert('Server name is required');
      return;
    }
    
    if (!formData.minecraft_instance_id) {
      alert('Please select a Minecraft instance');
      return;
    }
    
    if (!formData.docker_connection_id) {
      alert('Please select a Docker connection');
      return;
    }

    const request: ServerDeploymentRequest = {
      name: formData.name.trim(),
      minecraft_instance_id: formData.minecraft_instance_id,
      docker_connection_id: formData.docker_connection_id,
      port: formData.port,
      max_players: formData.max_players,
      memory_limit: formData.memory_limit,
      environment_vars: formData.environment_vars,
      auto_start: formData.auto_start
    };

    onDeploy(request);
  };

  const handleAddEnvVar = () => {
    if (envVarInput.key.trim() && envVarInput.value.trim()) {
      setFormData(prev => ({
        ...prev,
        environment_vars: {
          ...prev.environment_vars,
          [envVarInput.key.trim()]: envVarInput.value.trim()
        }
      }));
      setEnvVarInput({ key: '', value: '' });
    }
  };

  const handleRemoveEnvVar = (key: string) => {
    setFormData(prev => {
      const newEnvVars = { ...prev.environment_vars };
      delete newEnvVars[key];
      return {
        ...prev,
        environment_vars: newEnvVars
      };
    });
  };

  const selectedInstanceData = instances.find(i => i.id === formData.minecraft_instance_id);

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
          {/* Basic Settings */}
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-white mb-3">Basic Settings</h3>
            
            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Server Name
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                placeholder="Enter server name"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Minecraft Instance
              </label>
              <select
                value={formData.minecraft_instance_id}
                onChange={(e) => setFormData(prev => ({ ...prev, minecraft_instance_id: e.target.value }))}
                className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                required
                disabled={!!selectedInstance}
              >
                <option value="">Select an instance</option>
                {instances.map((instance) => (
                  <option key={instance.id} value={instance.id}>
                    {instance.name} (Minecraft {instance.version})
                  </option>
                ))}
              </select>
              {selectedInstanceData && (
                <p className="text-xs text-primary-400 mt-1">
                  Game directory: {selectedInstanceData.gameDir}
                </p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Docker Connection
              </label>
              <select
                value={formData.docker_connection_id}
                onChange={(e) => setFormData(prev => ({ ...prev, docker_connection_id: e.target.value }))}
                className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                required
              >
                <option value="">Select a Docker connection</option>
                {dockerConnections.filter(c => c.is_connected).map((connection) => (
                  <option key={connection.id} value={connection.id}>
                    {connection.name} ({connection.host})
                  </option>
                ))}
              </select>
              {dockerConnections.filter(c => c.is_connected).length === 0 && (
                <p className="text-xs text-red-400 mt-1">
                  No Docker connections available. Please add one first.
                </p>
              )}
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-primary-300 mb-2">
                  Server Port
                </label>
                <input
                  type="number"
                  min="1024"
                  max="65535"
                  value={formData.port}
                  onChange={(e) => setFormData(prev => ({ ...prev, port: parseInt(e.target.value) || 25565 }))}
                  className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-primary-300 mb-2">
                  Max Players
                </label>
                <input
                  type="number"
                  min="1"
                  max="200"
                  value={formData.max_players}
                  onChange={(e) => setFormData(prev => ({ ...prev, max_players: parseInt(e.target.value) || 20 }))}
                  className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Memory Limit (MB)
              </label>
              <input
                type="number"
                min="512"
                max="32768"
                step="512"
                value={formData.memory_limit}
                onChange={(e) => setFormData(prev => ({ ...prev, memory_limit: parseInt(e.target.value) || 2048 }))}
                className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
              />
              <p className="text-xs text-primary-400 mt-1">
                Recommended: 2048MB for vanilla, 4096MB+ for modded servers
              </p>
            </div>

            <div className="flex items-center">
              <input
                type="checkbox"
                id="auto_start"
                checked={formData.auto_start}
                onChange={(e) => setFormData(prev => ({ ...prev, auto_start: e.target.checked }))}
                className="mr-2"
              />
              <label htmlFor="auto_start" className="text-sm text-primary-300">
                Start server automatically after deployment
              </label>
            </div>
          </div>

          {/* Advanced Settings */}
          <div>
            <button
              type="button"
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center gap-2 text-secondary-400 hover:text-secondary-300 transition-colors"
            >
              <Settings className="w-4 h-4" />
              Advanced Settings
            </button>
            
            {showAdvanced && (
              <div className="mt-4 space-y-4">
                <div>
                  <label className="block text-sm font-medium text-primary-300 mb-2">
                    Environment Variables
                  </label>
                  <div className="space-y-2">
                    <div className="flex gap-2">
                      <input
                        type="text"
                        placeholder="Variable name"
                        value={envVarInput.key}
                        onChange={(e) => setEnvVarInput(prev => ({ ...prev, key: e.target.value }))}
                        className="flex-1 px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                      />
                      <input
                        type="text"
                        placeholder="Value"
                        value={envVarInput.value}
                        onChange={(e) => setEnvVarInput(prev => ({ ...prev, value: e.target.value }))}
                        className="flex-1 px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
                      />
                      <button
                        type="button"
                        onClick={handleAddEnvVar}
                        className="px-3 py-2 bg-secondary-600 hover:bg-secondary-700 text-white rounded-lg transition-colors"
                      >
                        Add
                      </button>
                    </div>
                    
                    {Object.entries(formData.environment_vars).map(([key, value]) => (
                      <div key={key} className="flex items-center justify-between bg-primary-700/50 px-3 py-2 rounded-lg">
                        <span className="text-sm text-white">{key} = {value}</span>
                        <button
                          type="button"
                          onClick={() => handleRemoveEnvVar(key)}
                          className="text-red-400 hover:text-red-300 transition-colors"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>

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