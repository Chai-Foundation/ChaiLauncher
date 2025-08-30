import React from 'react';

interface ConnectionFormProps {
  formData: {
    name: string;
    host: string;
    port: number | undefined;
    connection_type: 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm';
  };
  setFormData: React.Dispatch<React.SetStateAction<{
    name: string;
    host: string;
    port: number | undefined;
    connection_type: 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm';
  }>>;
  getConnectionTypeLabel: (type: string) => string;
}

export const ConnectionForm: React.FC<ConnectionFormProps> = ({
  formData,
  setFormData,
  getConnectionTypeLabel
}) => {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-primary-300 mb-2">
          Connection Name
        </label>
        <input
          type="text"
          value={formData.name}
          onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
          className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
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
          onChange={(e) => setFormData(prev => ({ 
            ...prev, 
            connection_type: e.target.value as typeof formData.connection_type
          }))}
          className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
        >
          {(['local', 'windows_named_pipe', 'unix_socket', 'remote', 'swarm'] as const).map(type => (
            <option key={type} value={type}>
              {getConnectionTypeLabel(type)}
            </option>
          ))}
        </select>
      </div>

      {(formData.connection_type === 'remote' || formData.connection_type === 'swarm') && (
        <>
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Host
            </label>
            <input
              type="text"
              value={formData.host}
              onChange={(e) => setFormData(prev => ({ ...prev, host: e.target.value }))}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
              placeholder="docker.example.com"
              required={formData.connection_type === 'remote' || formData.connection_type === 'swarm'}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Port
            </label>
            <input
              type="number"
              min="1"
              max="65535"
              value={formData.port || ''}
              onChange={(e) => setFormData(prev => ({ 
                ...prev, 
                port: e.target.value ? parseInt(e.target.value) : undefined 
              }))}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
              placeholder="2376"
            />
            <p className="text-xs text-primary-400 mt-1">
              Default: 2376 (TLS) or 2375 (unencrypted)
            </p>
          </div>
        </>
      )}

      {formData.connection_type === 'local' && (
        <div className="bg-primary-700/30 p-3 rounded-lg">
          <p className="text-sm text-primary-300">
            Local Docker Desktop connection. Make sure Docker Desktop is running.
          </p>
        </div>
      )}

      {formData.connection_type === 'unix_socket' && (
        <div className="bg-primary-700/30 p-3 rounded-lg">
          <p className="text-sm text-primary-300">
            Unix socket connection (typically /var/run/docker.sock).
          </p>
        </div>
      )}
    </div>
  );
};