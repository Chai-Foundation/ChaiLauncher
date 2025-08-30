import React from 'react';
import { MinecraftInstance } from '../../types/minecraft';
import { DockerConnection } from '../../types/servers';

interface BasicServerSettingsProps {
  formData: {
    name: string;
    minecraft_instance_id: string;
    docker_connection_id: string;
    port: number;
    max_players: number;
    memory_limit: number;
    auto_start: boolean;
  };
  setFormData: React.Dispatch<React.SetStateAction<any>>;
  instances: MinecraftInstance[];
  dockerConnections: DockerConnection[];
  selectedInstanceData?: MinecraftInstance;
  selectedInstance?: MinecraftInstance | null;
}

export const BasicServerSettings: React.FC<BasicServerSettingsProps> = ({
  formData,
  setFormData,
  instances,
  dockerConnections,
  selectedInstanceData,
  selectedInstance
}) => {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold text-white mb-3">Basic Settings</h3>
      
      <div>
        <label className="block text-sm font-medium text-primary-300 mb-2">
          Server Name
        </label>
        <input
          type="text"
          value={formData.name}
          onChange={(e) => setFormData((prev: any) => ({ ...prev, name: e.target.value }))}
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
          onChange={(e) => setFormData((prev: any) => ({ ...prev, minecraft_instance_id: e.target.value }))}
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
          onChange={(e) => setFormData((prev: any) => ({ ...prev, docker_connection_id: e.target.value }))}
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
            onChange={(e) => setFormData((prev: any) => ({ ...prev, port: parseInt(e.target.value) || 25565 }))}
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
            onChange={(e) => setFormData((prev: any) => ({ ...prev, max_players: parseInt(e.target.value) || 20 }))}
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
          onChange={(e) => setFormData((prev: any) => ({ ...prev, memory_limit: parseInt(e.target.value) || 2048 }))}
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
          onChange={(e) => setFormData((prev: any) => ({ ...prev, auto_start: e.target.checked }))}
          className="mr-2"
        />
        <label htmlFor="auto_start" className="text-sm text-primary-300">
          Start server automatically after deployment
        </label>
      </div>
    </div>
  );
};