import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Server, 
  Play, 
  Square, 
  Trash2, 
  Settings, 
  Plus, 
  Container,
  RefreshCw,
  Monitor,
  Users,
  HardDrive,
  Activity,
  Terminal
} from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';
import { 
  ServerInstance, 
  DockerConnection, 
  ServerDeploymentRequest,
  ServerStatus 
} from '../types/servers';
import DeployServerModal from './DeployServerModal';
import DockerConnectionModal from './DockerConnectionModal';
import ServerLogsModal from './ServerLogsModal';

interface ServersViewProps {
  instances: MinecraftInstance[];
}

const ServersView: React.FC<ServersViewProps> = ({ instances }) => {
  const [servers, setServers] = useState<ServerInstance[]>([]);
  const [dockerConnections, setDockerConnections] = useState<DockerConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [showDeployModal, setShowDeployModal] = useState(false);
  const [showConnectionModal, setShowConnectionModal] = useState(false);
  const [showLogsModal, setShowLogsModal] = useState(false);
  const [selectedServer, setSelectedServer] = useState<ServerInstance | null>(null);
  const [selectedInstance, setSelectedInstance] = useState<MinecraftInstance | null>(null);

  useEffect(() => {
    loadServers();
    // Load Docker connections from persistent config
    const loadDockerConnections = async () => {
      try {
        const connections = await invoke<DockerConnection[]>("get_docker_connections");
        setDockerConnections(connections);
      } catch (error) {
        console.error("Failed to load Docker connections:", error);
      }
    };
    loadDockerConnections();
  }, []);

  // Separate effect for periodic status refresh
  useEffect(() => {
    if (servers.length === 0) return;

    // Set up periodic status refresh every 10 seconds
    const statusInterval = setInterval(() => {
      refreshServerStatuses();
    }, 10000);

    return () => {
      clearInterval(statusInterval);
    };
  }, [servers.length]); // Only depend on the number of servers, not the full array

  const loadServers = async () => {
    try {
      setLoading(true);
      const serversData = await invoke<ServerInstance[]>('get_servers');
      setServers(serversData);
      // After loading servers, refresh their status from Docker
      refreshServerStatuses(serversData);
    } catch (error) {
      console.error('Failed to load servers:', error);
    } finally {
      setLoading(false);
    }
  };

  const refreshServerStatuses = async (serverList?: ServerInstance[]) => {
    const serversToCheck = serverList || servers;
    if (serversToCheck.length === 0) return;

    try {
      // Check status for each server
      const statusPromises = serversToCheck.map(async (server) => {
        try {
          const status = await invoke<ServerStatus>('get_server_status', { serverId: server.id });
          return { id: server.id, status };
        } catch (error) {
          console.error(`Failed to get status for server ${server.id}:`, error);
          return { id: server.id, status: 'unknown' as ServerStatus };
        }
      });

      const statusResults = await Promise.all(statusPromises);
      
      // Update server statuses in state
      setServers(prev => 
        prev.map(server => {
          const statusUpdate = statusResults.find(result => result.id === server.id);
          return statusUpdate ? { ...server, status: statusUpdate.status } : server;
        })
      );
    } catch (error) {
      console.error('Failed to refresh server statuses:', error);
    }
  };

  const handleStartServer = async (server: ServerInstance) => {
    try {
      await invoke('start_server', { serverId: server.id });
      // Update local state immediately for responsive UI
      setServers(prev => 
        prev.map(s => s.id === server.id ? { ...s, status: 'starting' as ServerStatus } : s)
      );
      // Refresh status after a short delay to get actual Docker status
      setTimeout(() => refreshServerStatuses(), 2000);
    } catch (error) {
      console.error('Failed to start server:', error);
      alert(`Failed to start server: ${error}`);
    }
  };

  const handleStopServer = async (server: ServerInstance) => {
    try {
      await invoke('stop_server', { serverId: server.id });
      setServers(prev => 
        prev.map(s => s.id === server.id ? { ...s, status: 'stopping' as ServerStatus } : s)
      );
      // Refresh status after a short delay to get actual Docker status
      setTimeout(() => refreshServerStatuses(), 2000);
    } catch (error) {
      console.error('Failed to stop server:', error);
      alert(`Failed to stop server: ${error}`);
    }
  };

  const handleRemoveServer = async (server: ServerInstance) => {
    if (!confirm(`Are you sure you want to remove server "${server.name}"? This will delete the container.`)) {
      return;
    }

    try {
      await invoke('remove_server', { serverId: server.id });
      setServers(prev => prev.filter(s => s.id !== server.id));
    } catch (error) {
      console.error('Failed to remove server:', error);
    }
  };

  const handleDeployServer = async (request: ServerDeploymentRequest) => {
    try {
      console.log('Deploying server with request:', request);
      
      const instance = instances.find(i => i.id === request.minecraft_instance_id);
      if (!instance) {
        throw new Error('Minecraft instance not found');
      }

      console.log('Found instance:', instance);

      const result = await invoke('deploy_minecraft_server', { 
        deploymentRequest: request,
        minecraftInstance: instance 
      });
      
      console.log('Deploy server result:', result);
      
      setShowDeployModal(false);
      await loadServers();
      // Refresh status after deployment
      setTimeout(() => refreshServerStatuses(), 3000);
      
      // Show success message
      alert(`Server "${request.name}" deployed successfully!`);
    } catch (error) {
      console.error('Failed to deploy server:', error);
      alert(`Failed to deploy server: ${error}`);
    }
  };

  const handleAddDockerConnection = async (connection: DockerConnection) => {
    try {
      await invoke('add_docker_connection', { connection });
      setDockerConnections(prev => [...prev, { ...connection, is_connected: true }]);
      setShowConnectionModal(false);
    } catch (error) {
      console.error('Failed to add Docker connection:', error);
    }
  };

  const getStatusColor = (status: ServerStatus): string => {
    switch (status) {
      case 'running': return 'text-secondary-400';
      case 'starting': return 'text-secondary-500';
      case 'stopping': return 'text-secondary-500';
      case 'stopped': return 'text-primary-400';
      case 'failed': return 'text-secondary-600';
      default: return 'text-primary-400';
    }
  };

  const getStatusIcon = (status: ServerStatus) => {
    switch (status) {
      case 'running': return <Activity className="w-4 h-4" />;
      case 'starting': case 'stopping': return <div className="w-4 h-4 rounded-full border-2 border-current border-t-transparent animate-spin" />;
      case 'stopped': return <Square className="w-4 h-4" />;
      case 'failed': return <Trash2 className="w-4 h-4" />;
      default: return <Monitor className="w-4 h-4" />;
    }
  };

  // Group servers by instance
  const serversByInstance = servers.reduce((acc, server) => {
    const instanceId = server.minecraft_instance_id;
    if (!acc[instanceId]) {
      acc[instanceId] = [];
    }
    acc[instanceId].push(server);
    return acc;
  }, {} as Record<string, ServerInstance[]>);

  if (loading) {
    return (
      <div className="flex-1 p-6 flex items-center justify-center">
        <div className="text-white text-lg">Loading servers...</div>
      </div>
    );
  }

  return (
    <div className="flex-1 p-6 text-white">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-secondary-200 via-white to-secondary-200 bg-clip-text text-transparent mb-2">
            Server Management
          </h1>
          <p className="text-primary-300">Deploy and manage Minecraft servers with Docker</p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => refreshServerStatuses()}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
            title="Refresh server statuses"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh Status
          </button>
        </div>
      </div>

      {/* Docker Connections Section */}
      <div className="mb-6">
        <div className="bg-primary-800/30 backdrop-blur-sm rounded-xl p-6 border border-secondary-600/20">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 bg-secondary-600/20 rounded-lg flex items-center justify-center">
                <Container className="w-6 h-6 text-secondary-400" />
              </div>
              <div>
                <h2 className="text-xl font-semibold">Docker Connections</h2>
                <p className="text-primary-300 text-sm">Manage Docker environments for server deployment</p>
              </div>
            </div>
            <button
              onClick={() => setShowConnectionModal(true)}
              className="flex items-center gap-2 px-3 py-2 bg-secondary-600/20 hover:bg-secondary-600/30 rounded-lg transition-colors text-secondary-400"
            >
              <Plus className="w-4 h-4" />
              Add Connection
            </button>
          </div>
          
          {dockerConnections.length === 0 ? (
            <div className="text-center py-8 text-primary-400">
              <Container className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No Docker connections configured</p>
              <p className="text-sm mt-1">Add a connection to deploy servers</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {dockerConnections.map((connection) => (
                <div key={connection.id} className="bg-primary-700/30 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <h3 className="font-semibold">{connection.name}</h3>
                    <div className={`w-3 h-3 rounded-full ${connection.is_connected ? 'bg-secondary-400' : 'bg-secondary-500/50'}`} />
                  </div>
                  <p className="text-sm text-primary-300">{connection.host}</p>
                  <p className="text-xs text-primary-400 mt-1 capitalize">{connection.connection_type}</p>
                  <div className="mt-2 text-xs">
                    <span className={`${connection.is_connected ? 'text-secondary-400' : 'text-secondary-500/70'}`}>
                      {connection.is_connected ? 'Connected' : 'Disconnected'}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="space-y-6">
        {instances.map((instance) => {
          const instanceServers = serversByInstance[instance.id] || [];
          
          return (
            <div key={instance.id} className="bg-primary-800/30 backdrop-blur-sm rounded-xl p-6 border border-secondary-600/20">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="w-12 h-12 bg-secondary-600/20 rounded-lg flex items-center justify-center">
                    <Server className="w-6 h-6 text-secondary-400" />
                  </div>
                  <div>
                    <h2 className="text-xl font-semibold">{instance.name}</h2>
                    <p className="text-primary-300 text-sm">Minecraft {instance.version}</p>
                  </div>
                </div>
                <button
                  onClick={() => {
                    setSelectedInstance(instance);
                    setShowDeployModal(true);
                  }}
                  className="flex items-center gap-2 px-3 py-2 bg-secondary-600/20 hover:bg-secondary-600/30 rounded-lg transition-colors text-secondary-400"
                >
                  <Plus className="w-4 h-4" />
                  Deploy Server
                </button>
              </div>

              {instanceServers.length === 0 ? (
                <div className="text-center py-8 text-primary-400">
                  <Server className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No servers deployed for this instance</p>
                  <p className="text-sm mt-1">Click "Deploy Server" to create one</p>
                </div>
              ) : (
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  {instanceServers.map((server) => (
                    <div key={server.id} className="bg-primary-700/30 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <div className={`${getStatusColor(server.status)}`}>
                            {getStatusIcon(server.status)}
                          </div>
                          <h3 className="font-semibold">{server.name}</h3>
                        </div>
                        <div className="flex items-center gap-1">
                          {server.status === 'running' && (
                            <button
                              onClick={() => {
                                setSelectedServer(server);
                                setShowLogsModal(true);
                              }}
                              className="p-2 text-primary-400 hover:text-white hover:bg-primary-600/50 rounded transition-colors"
                              title="View Logs"
                            >
                              <Terminal className="w-4 h-4" />
                            </button>
                          )}
                          <button
                            onClick={() => handleStartServer(server)}
                            disabled={server.status === 'running' || server.status === 'starting'}
                            className="p-2 text-secondary-400 hover:text-secondary-300 hover:bg-secondary-600/20 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            title="Start Server"
                          >
                            <Play className="w-4 h-4" />
                          </button>
                          <button
                            onClick={() => handleStopServer(server)}
                            disabled={server.status === 'stopped' || server.status === 'stopping'}
                            className="p-2 text-secondary-500 hover:text-secondary-400 hover:bg-secondary-600/20 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            title="Stop Server"
                          >
                            <Square className="w-4 h-4" />
                          </button>
                          <button
                            onClick={() => handleRemoveServer(server)}
                            className="p-2 text-secondary-600 hover:text-secondary-500 hover:bg-secondary-600/20 rounded transition-colors"
                            title="Remove Server"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>

                      <div className="space-y-3 text-sm">
                        <div className="grid grid-cols-2 gap-4">
                          <div>
                            <div className="flex items-center gap-2 text-primary-300 mb-1">
                              <Monitor className="w-4 h-4" />
                              Port: {server.port}
                            </div>
                            <div className="flex items-center gap-2 text-primary-300">
                              <Users className="w-4 h-4" />
                              Max Players: {server.max_players}
                            </div>
                          </div>
                          <div>
                            <div className="flex items-center gap-2 text-primary-300 mb-1">
                              <HardDrive className="w-4 h-4" />
                              RAM: {server.memory_limit} MB
                            </div>
                            <div className="text-xs text-primary-400">
                              {server.last_started ? 
                                `Last started: ${new Date(server.last_started).toLocaleString()}` : 
                                'Never started'
                              }
                            </div>
                          </div>
                        </div>
                        
                        {/* Docker Connection Info */}
                        <div className="pt-2 border-t border-primary-600/30">
                          <div className="flex items-center gap-2 text-primary-300">
                            <Container className="w-4 h-4" />
                            <span>Docker: </span>
                            {(() => {
                              const connection = dockerConnections.find(c => c.id === server.docker_connection_id);
                              return connection ? (
                                <span className="flex items-center gap-2">
                                  <span>{connection.name}</span>
                                  <div className={`w-2 h-2 rounded-full ${connection.is_connected ? 'bg-secondary-400' : 'bg-secondary-500/50'}`} />
                                </span>
                              ) : (
                                <span className="text-secondary-500/70">Connection not found</span>
                              );
                            })()}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}

        {instances.length === 0 && (
          <div className="text-center py-12 text-primary-400">
            <Server className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <h3 className="text-xl font-semibold mb-2">No Minecraft Instances</h3>
            <p>You need to create Minecraft instances before you can deploy servers.</p>
            <p className="text-sm mt-1">Go to the Instances tab to create one.</p>
          </div>
        )}
      </div>

      <DeployServerModal
        isOpen={showDeployModal}
        onClose={() => {
          setShowDeployModal(false);
          setSelectedInstance(null);
        }}
        onDeploy={handleDeployServer}
        instances={instances}
        dockerConnections={dockerConnections}
        selectedInstance={selectedInstance}
      />

      <DockerConnectionModal
        isOpen={showConnectionModal}
        onClose={() => setShowConnectionModal(false)}
        onAdd={handleAddDockerConnection}
      />

      {selectedServer && (
        <ServerLogsModal
          isOpen={showLogsModal}
          onClose={() => {
            setShowLogsModal(false);
            setSelectedServer(null);
          }}
          server={selectedServer}
        />
      )}
    </div>
  );
};

export default ServersView;