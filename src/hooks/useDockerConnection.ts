import { useState } from 'react';
import { DockerConnection } from '../types/servers';
import { invoke } from '@tauri-apps/api/core';

export const useDockerConnection = () => {
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: undefined as number | undefined,
    connection_type: 'local' as 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm'
  });

  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const resetForm = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: undefined,
      connection_type: 'local'
    });
    setTestResult(null);
  };

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);
    
    try {
      // Create a temporary connection object for testing
      const tempConnection: DockerConnection = {
        id: 'temp-test',
        name: formData.name.trim() || 'Test Connection',
        host: formData.host,
        port: formData.port,
        connection_type: formData.connection_type,
        is_connected: false
      };

      await invoke('test_docker_connection', { connection: tempConnection });
      
      setTestResult({
        success: true,
        message: 'Connection successful! Docker daemon is reachable.'
      });
    } catch (error) {
      console.error('Docker connection test failed:', error);
      setTestResult({
        success: false,
        message: `Connection failed: ${error}`
      });
    } finally {
      setTesting(false);
    }
  };

  const validateAndCreateConnection = (): DockerConnection | string => {
    if (!formData.name.trim()) {
      return 'Connection name is required';
    }

    return {
      id: `docker-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      name: formData.name.trim(),
      host: formData.host,
      port: formData.port,
      connection_type: formData.connection_type,
      is_connected: false // Will be set to true after successful test
    };
  };

  const getConnectionTypeLabel = (type: string) => {
    switch (type) {
      case 'local':
        return 'Local Docker Desktop';
      case 'windows_named_pipe':
        return 'Windows Named Pipe';
      case 'unix_socket':
        return 'Unix Socket';
      case 'remote':
        return 'Remote Docker Host';
      case 'swarm':
        return 'Docker Swarm';
      default:
        return type;
    }
  };

  return {
    formData,
    setFormData,
    testing,
    testResult,
    resetForm,
    testConnection,
    validateAndCreateConnection,
    getConnectionTypeLabel
  };
};