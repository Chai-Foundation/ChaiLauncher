export interface DockerConnection {
  id: string;
  name: string;
  host: string;
  port?: number;
  connection_type: 'local' | 'windows_named_pipe' | 'unix_socket' | 'remote' | 'swarm';
  is_connected: boolean;
}

export interface ServerInstance {
  id: string;
  name: string;
  minecraft_instance_id: string;
  docker_connection_id: string;
  container_id?: string;
  status: ServerStatus;
  port: number;
  max_players: number;
  memory_limit: number; // in MB
  created_at: string;
  last_started?: string;
  environment_vars: Record<string, string>;
}

export type ServerStatus = 
  | 'stopped' 
  | 'starting' 
  | 'running' 
  | 'stopping' 
  | 'failed' 
  | 'unknown';

export interface ServerDeploymentRequest {
  name: string;
  minecraft_instance_id: string;
  docker_connection_id: string;
  port: number;
  max_players: number;
  memory_limit: number;
  environment_vars: Record<string, string>;
  auto_start: boolean;
}

export interface ServerStats {
  cpu_usage: number;
  memory_usage: number;
  memory_limit: number;
  network_in: number;
  network_out: number;
  uptime: number;
}

export interface LogEntry {
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

export interface DockerImage {
  id: string;
  repository: string;
  tag: string;
  size: number;
  created: string;
}