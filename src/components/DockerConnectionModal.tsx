import React, { useState } from 'react';
import { X, Container, TestTube, CheckCircle } from 'lucide-react';
import { DockerConnection } from '../types/servers';
import { invoke } from '@tauri-apps/api/core';

interface DockerConnectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (connection: DockerConnection) => void;
}

const DockerConnectionModal: React.FC<DockerConnectionModalProps> = ({ isOpen, onClose, onAdd }) => {
  const [formData, setFormData] = useState({
    name: '',
    host: 'localhost',
    port: undefined as number | undefined,
    connection_type: 'local' as 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm'
  });

  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      alert('Connection name is required');
      return;
    }

    const connection: DockerConnection = {
      id: `docker-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      name: formData.name.trim(),
      host: formData.host,
      port: formData.port,
      connection_type: formData.connection_type,
      is_connected: false // Will be set to true after successful test
    };

    onAdd(connection);
    onClose();
    resetForm();
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);

    try {
      const testConnection: DockerConnection = {
        id: 'test',
        name: formData.name.trim() || 'Test Connection',
        host: formData.host,
        port: formData.port,
        connection_type: formData.connection_type,
        is_connected: false
      };

      const success = await invoke<boolean>('test_docker_connection', { connection: testConnection });
      
      if (success) {
        setTestResult({ success: true, message: 'Connection successful!' });
      } else {
        setTestResult({ success: false, message: 'Connection failed' });
      }
    } catch (error) {
      setTestResult({ 
        success: false, 
        message: error instanceof Error ? error.message : 'Connection failed' 
      });
    } finally {
      setTesting(false);
    }
  };

  const resetForm = () => {
    setFormData({
      name: '',
      host: 'localhost',
      port: undefined,
      connection_type: 'local'
    });
    setTestResult(null);
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-primary-800 rounded-xl p-6 max-w-lg w-full">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-blue-600/20 rounded-lg flex items-center justify-center">
              <Container className="w-5 h-5 text-blue-400" />
            </div>
            <h2 className="text-xl font-bold text-white">Add Docker Connection</h2>
          </div>
          <button
            onClick={handleClose}
            className="text-primary-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Connection Name
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-blue-500 focus:outline-none"
              placeholder="My Docker Server"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Connection Type
            </label>
            <select
              value={formData.connection_type}
              onChange={(e) => {
                const type = e.target.value as 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm';
                setFormData(prev => ({ 
                  ...prev, 
                  connection_type: type,
                  host: ['local', 'windows_named_pipe', 'unix_socket'].includes(type) ? 'localhost' : prev.host,
                  port: ['local', 'windows_named_pipe', 'unix_socket'].includes(type) ? undefined : (prev.port || 2376)
                }));
              }}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-blue-500 focus:outline-none"
            >
              <option value="local">Local Docker (Auto-detect)</option>
              <option value="windows_named_pipe">Windows Named Pipe</option>
              <option value="unix_socket">Unix Socket</option>
              <option value="remote">Remote Docker</option>
              <option value="swarm">Docker Swarm</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Host
            </label>
            <input
              type="text"
              value={formData.host}
              onChange={(e) => setFormData(prev => ({ ...prev, host: e.target.value }))}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-blue-500 focus:outline-none"
              placeholder="localhost"
              disabled={['local', 'windows_named_pipe', 'unix_socket'].includes(formData.connection_type)}
              required
            />
          </div>

          {!['local', 'windows_named_pipe', 'unix_socket'].includes(formData.connection_type) && (
            <div>
              <label className="block text-sm font-medium text-primary-300 mb-2">
                Port
              </label>
              <input
                type="number"
                min="1"
                max="65535"
                value={formData.port || ''}
                onChange={(e) => setFormData(prev => ({ ...prev, port: e.target.value ? parseInt(e.target.value) : undefined }))}
                className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-blue-500 focus:outline-none"
                placeholder="2376"
              />
            </div>
          )}

          {/* Connection Info */}
          <div className="bg-primary-700/30 rounded-lg p-4 text-sm">
            <h4 className="font-medium text-primary-200 mb-2">Connection Information:</h4>
            <ul className="text-primary-300 space-y-1">
              {formData.connection_type === 'local' && (
                <li>• Auto-detects Docker Desktop or local Docker daemon</li>
              )}
              {formData.connection_type === 'windows_named_pipe' && (
                <li>• Connects via Windows named pipe (\\.\pipe\docker_engine)</li>
              )}
              {formData.connection_type === 'unix_socket' && (
                <li>• Connects via Unix socket (/var/run/docker.sock)</li>
              )}
              {formData.connection_type === 'remote' && (
                <li>• Connects to a remote Docker daemon via TCP</li>
              )}
              {formData.connection_type === 'swarm' && (
                <li>• Connects to a Docker Swarm manager node via TCP</li>
              )}
              <li>• Make sure Docker is running and accessible</li>
              {!['local', 'windows_named_pipe', 'unix_socket'].includes(formData.connection_type) && (
                <li>• Ensure the Docker daemon accepts TCP connections</li>
              )}
            </ul>
          </div>

          {/* Test Connection */}
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={handleTestConnection}
              disabled={testing || !formData.name.trim()}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-600/50 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
            >
              {testing ? (
                <>
                  <div className="w-4 h-4 rounded-full border-2 border-current border-t-transparent animate-spin" />
                  Testing...
                </>
              ) : (
                <>
                  <TestTube className="w-4 h-4" />
                  Test Connection
                </>
              )}
            </button>
            
            {testResult && (
              <div className={`flex items-center gap-2 ${testResult.success ? 'text-green-400' : 'text-red-400'}`}>
                {testResult.success && <CheckCircle className="w-4 h-4" />}
                <span className="text-sm">{testResult.message}</span>
              </div>
            )}
          </div>

          <div className="flex justify-end gap-3 pt-4 border-t border-primary-600">
            <button
              type="button"
              onClick={handleClose}
              className="px-4 py-2 text-primary-300 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!testResult?.success}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-600/50 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
            >
              Add Connection
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default DockerConnectionModal;