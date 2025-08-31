import { useState, useEffect, useCallback } from 'react';
import { MinecraftInstance } from '../types/minecraft';
import { DockerConnection, ServerDeploymentRequest } from '../types/servers';

export const useServerDeployment = (
  instances: MinecraftInstance[],
  dockerConnections: DockerConnection[],
  selectedInstance?: MinecraftInstance | null
) => {
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
      setFormData(prev => {
        // Only update if the instance ID actually changed or name is empty
        if (prev.minecraft_instance_id !== selectedInstance.id) {
          return {
            ...prev,
            minecraft_instance_id: selectedInstance.id,
            name: `${selectedInstance.name}-server`
          };
        }
        // If instance is the same but name is empty, set default name
        if (!prev.name) {
          return {
            ...prev,
            name: `${selectedInstance.name}-server`
          };
        }
        return prev;
      });
    }
  }, [selectedInstance?.id, selectedInstance?.name]); // Only depend on specific properties

  const resetForm = useCallback(() => {
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
  }, [selectedInstance?.id]);

  const addEnvironmentVariable = useCallback(() => {
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
  }, [envVarInput.key, envVarInput.value]);

  const removeEnvironmentVariable = useCallback((key: string) => {
    setFormData(prev => {
      const newEnvVars = { ...prev.environment_vars };
      delete newEnvVars[key];
      return {
        ...prev,
        environment_vars: newEnvVars
      };
    });
  }, []);

  const validateForm = useCallback((): string | null => {
    if (!formData.name.trim()) {
      return 'Server name is required';
    }
    
    if (!formData.minecraft_instance_id) {
      return 'Please select a Minecraft instance';
    }
    
    if (!formData.docker_connection_id) {
      return 'Please select a Docker connection';
    }

    return null;
  }, [formData.name, formData.minecraft_instance_id, formData.docker_connection_id]);

  const createDeploymentRequest = useCallback((): ServerDeploymentRequest => {
    return {
      name: formData.name.trim(),
      minecraft_instance_id: formData.minecraft_instance_id,
      docker_connection_id: formData.docker_connection_id,
      port: formData.port,
      max_players: formData.max_players,
      memory_limit: formData.memory_limit,
      environment_vars: formData.environment_vars,
      auto_start: formData.auto_start
    };
  }, [formData]);

  const selectedInstanceData = instances.find(i => i.id === formData.minecraft_instance_id);

  return {
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
  };
};