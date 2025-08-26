use super::types::*;
use crate::minecraft::MinecraftInstance;
use crate::storage::StorageManager;
use bollard::{Docker, API_DEFAULT_VERSION};
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, 
    StopContainerOptions, ListContainersOptions
};
use bollard::models::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum};
use bollard::image::ListImagesOptions;
use std::collections::HashMap;
use uuid;

pub struct DockerManager {
    connections: HashMap<String, Docker>,
    storage: StorageManager,
}

impl DockerManager {
    pub async fn new() -> Result<Self, String> {
        let storage = StorageManager::new().await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;
        
        let mut manager = Self {
            connections: HashMap::new(),
            storage,
        };
        
        // Load and reconnect to saved Docker connections
        manager.reconnect_saved_connections().await?;
        
        Ok(manager)
    }

    /// Reconnect to all saved Docker connections
    async fn reconnect_saved_connections(&mut self) -> Result<(), String> {
        let connections = self.storage.get_docker_connections().iter().cloned().cloned().collect::<Vec<_>>();
        
        for connection in connections {
            if let Ok(docker) = self.create_docker_connection(&connection).await {
                self.connections.insert(connection.id.clone(), docker);
                // Update connection status to connected in storage
                let mut updated_connection = connection;
                updated_connection.is_connected = true;
                if let Err(e) = self.storage.update_docker_connection(updated_connection).await {
                    eprintln!("Failed to update connection status: {}", e);
                }
            } else {
                // Mark connection as disconnected if we can't connect
                let mut updated_connection = connection;
                updated_connection.is_connected = false;
                if let Err(e) = self.storage.update_docker_connection(updated_connection).await {
                    eprintln!("Failed to update connection status: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Create a Docker connection based on the connection type
    async fn create_docker_connection(&self, connection: &DockerConnection) -> Result<Docker, String> {
        match connection.connection_type {
            DockerConnectionType::Local => {
                self.connect_local_docker()
                    .map_err(|e| format!("Failed to connect to local Docker: {}", e))
            },
            DockerConnectionType::WindowsNamedPipe => {
                #[cfg(target_os = "windows")]
                {
                    Docker::connect_with_named_pipe_defaults()
                        .map_err(|e| format!("Failed to connect via Windows named pipe: {}", e))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err("Windows named pipe connection is only available on Windows".to_string())
                }
            },
            DockerConnectionType::UnixSocket => {
                #[cfg(not(target_os = "windows"))]
                {
                    Docker::connect_with_socket_defaults()
                        .map_err(|e| format!("Failed to connect via Unix socket: {}", e))
                }
                #[cfg(target_os = "windows")]
                {
                    Err("Unix socket connection is not available on Windows".to_string())
                }
            },
            DockerConnectionType::Remote => {
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to remote Docker: {}", e))
            },
            DockerConnectionType::Swarm => {
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to Docker Swarm: {}", e))
            }
        }
    }

    /// Test connection to Docker
    pub async fn test_connection(&self, connection: &DockerConnection) -> Result<bool, String> {
        let docker = self.create_docker_connection(connection).await?;

        // Test the connection by pinging
        match docker.ping().await {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Docker ping failed: {}", e)),
        }
    }

    /// Connect to local Docker with platform-specific socket handling
    fn connect_local_docker(&self) -> Result<Docker, bollard::errors::Error> {
        #[cfg(target_os = "windows")]
        {
            // On Windows, try named pipe first, then fallback to TCP
            if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
                return Ok(docker);
            }
            // Fallback to TCP for Windows (Docker Desktop)
            Docker::connect_with_local_defaults()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // On Linux/Unix, try Unix socket first, then fallback to TCP
            if let Ok(docker) = Docker::connect_with_socket_defaults() {
                return Ok(docker);
            }
            // Fallback to local defaults (usually TCP on localhost)
            Docker::connect_with_local_defaults()
        }
    }

    /// Add a Docker connection
    pub async fn add_connection(&mut self, mut connection: DockerConnection) -> Result<(), String> {
        // Test connection first
        if !self.test_connection(&connection).await? {
            return Err("Failed to establish Docker connection".to_string());
        }

        // Create Docker client
        let docker = self.create_docker_connection(&connection).await?;

        // Mark as connected and save to storage
        connection.is_connected = true;
        self.storage.add_docker_connection(connection.clone()).await
            .map_err(|e| format!("Failed to save Docker connection: {}", e))?;

        // Store active connection
        self.connections.insert(connection.id.clone(), docker);
        Ok(())
    }

    /// Deploy a server for a Minecraft instance
    pub async fn deploy_server(
        &mut self,
        request: ServerDeploymentRequest,
        minecraft_instance: &MinecraftInstance,
    ) -> Result<ServerInstance, String> {
        let docker = self.connections.get(&request.docker_connection_id)
            .ok_or("Docker connection not found")?;

        // Use itzg's minecraft-server image instead of building our own
        let image_name = "itzg/minecraft-server:latest".to_string();
        self.ensure_itzg_image(docker, &image_name).await?;

        // Prepare volume mounts for itzg's minecraft-server
        let game_dir = minecraft_instance.game_dir.to_string_lossy().to_string();
        let mut binds = vec![
            format!("{}:/data", game_dir),
        ];

        // Add logs volume (optional for itzg image)
        binds.push(format!("chai-server-logs-{}:/data/logs", request.name));

        // Create port bindings (Minecraft server + RCON)
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "25565/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(request.port.to_string()),
            }]),
        );
        // RCON port (25575 is default for itzg's image)
        port_bindings.insert(
            "25575/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".to_string()), // Only localhost for security
                host_port: Some((request.port + 10).to_string()), // Offset RCON port
            }]),
        );

        // Configure container
        let host_config = Some(HostConfig {
            binds: Some(binds),
            port_bindings: Some(port_bindings),
            memory: Some((request.memory_limit * 1024 * 1024).try_into().unwrap()), // Convert MB to bytes
            restart_policy: Some(RestartPolicy {
                name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            }),
            ..Default::default()
        });

        // Environment variables for itzg's minecraft-server
        let mut env_vars = vec![
            "EULA=TRUE".to_string(),
            format!("MAX_PLAYERS={}", request.max_players),
            format!("MOTD={}", request.name),
            format!("VERSION={}", minecraft_instance.version),
            format!("MEMORY={}M", request.memory_limit),
            "TYPE=VANILLA".to_string(),
            "ONLINE_MODE=TRUE".to_string(),
            // Enable RCON for command execution
            "ENABLE_RCON=true".to_string(),
            "RCON_PASSWORD=minecraft".to_string(), // Default password
            "RCON_PORT=25575".to_string(),
        ];
        
        // Add custom environment variables
        for (key, value) in &request.environment_vars {
            env_vars.push(format!("{}={}", key, value));
        }

        let config = Config {
            image: Some(image_name),
            env: Some(env_vars),
            host_config,
            exposed_ports: Some({
                let mut ports = HashMap::new();
                ports.insert("25565/tcp".to_string(), HashMap::new()); // Minecraft server port
                ports.insert("25575/tcp".to_string(), HashMap::new()); // RCON port
                ports
            }),
            ..Default::default()
        };

        // Create container
        let container_name = format!("chai-server-{}", request.name);
        let options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        let container = docker.create_container(Some(options), config)
            .await
            .map_err(|e| format!("Failed to create container: {}", e))?;

        // Start container if auto_start is enabled
        if request.auto_start {
            docker.start_container(&container.id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| format!("Failed to start container: {}", e))?;
        }

        // Create server instance
        let server = ServerInstance {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            minecraft_instance_id: request.minecraft_instance_id,
            docker_connection_id: request.docker_connection_id,
            container_id: Some(container.id),
            status: if request.auto_start { ServerStatus::Starting } else { ServerStatus::Stopped },
            port: request.port,
            max_players: request.max_players,
            memory_limit: request.memory_limit,
            created_at: chrono::Utc::now(),
            last_started: if request.auto_start { Some(chrono::Utc::now()) } else { None },
            environment_vars: request.environment_vars,
        };

        // Save server to persistent storage
        self.storage.add_server(server.clone()).await
            .map_err(|e| format!("Failed to save server: {}", e))?;

        Ok(server)
    }

    /// Start a server
    pub async fn start_server(&mut self, server_id: &str) -> Result<(), String> {
        let mut server = self.storage.get_server(server_id)
            .ok_or("Server not found")?.clone();

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            docker.start_container(container_id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| format!("Failed to start container: {}", e))?;

            server.status = ServerStatus::Starting;
            server.last_started = Some(chrono::Utc::now());
            
            // Save updated server status
            self.storage.update_server(server).await
                .map_err(|e| format!("Failed to update server status: {}", e))?;
        }

        Ok(())
    }

    /// Stop a server
    pub async fn stop_server(&mut self, server_id: &str) -> Result<(), String> {
        let mut server = self.storage.get_server(server_id)
            .ok_or("Server not found")?.clone();

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            let options = StopContainerOptions { t: 30 }; // 30 second timeout
            docker.stop_container(container_id, Some(options))
                .await
                .map_err(|e| format!("Failed to stop container: {}", e))?;

            server.status = ServerStatus::Stopping;
            
            // Save updated server status
            self.storage.update_server(server).await
                .map_err(|e| format!("Failed to update server status: {}", e))?;
        }

        Ok(())
    }

    /// Get server status
    pub async fn get_server_status(&mut self, server_id: &str) -> Result<ServerStatus, String> {
        let server = self.storage.get_server(server_id)
            .ok_or("Server not found")?;

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            let containers = docker.list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("id".to_string(), vec![container_id.clone()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| format!("Failed to list containers: {}", e))?;

            if let Some(container) = containers.first() {
                let status = match container.state.as_deref() {
                    Some("running") => ServerStatus::Running,
                    Some("exited") => ServerStatus::Stopped,
                    Some("created") => ServerStatus::Stopped,
                    Some("restarting") => ServerStatus::Starting,
                    _ => ServerStatus::Unknown,
                };

                // Update status in storage if it's different
                if status != server.status {
                    let mut updated_server = server.clone();
                    updated_server.status = status.clone();
                    if let Err(e) = self.storage.update_server(updated_server).await {
                        eprintln!("Failed to update server status in storage: {}", e);
                    }
                }

                Ok(status)
            } else {
                Ok(ServerStatus::Unknown)
            }
        } else {
            Ok(ServerStatus::Unknown)
        }
    }

    /// Remove a server
    pub async fn remove_server(&mut self, server_id: &str) -> Result<(), String> {
        let server = self.storage.get_server(server_id)
            .ok_or("Server not found")?.clone();

        let container_id = server.container_id.clone();
        let docker_connection_id = server.docker_connection_id.clone();

        // Handle Docker container cleanup if needed
        if let Some(container_id) = container_id {
            if let Some(docker) = self.connections.get(&docker_connection_id) {
                // Stop the server first
                let stop_options = StopContainerOptions { t: 30 };
                let _ = docker.stop_container(&container_id, Some(stop_options)).await;
                
                // Wait a moment for container to stop
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Remove container
                let options = RemoveContainerOptions {
                    force: true,
                    v: true, // Remove volumes
                    ..Default::default()
                };

                docker.remove_container(&container_id, Some(options))
                    .await
                    .map_err(|e| format!("Failed to remove container: {}", e))?;
            }
        }

        // Remove from storage
        self.storage.remove_server(server_id).await
            .map_err(|e| format!("Failed to remove server from storage: {}", e))?;

        Ok(())
    }

    /// Get all servers
    pub fn get_servers(&self) -> Vec<&ServerInstance> {
        self.storage.get_servers()
    }

    /// Get servers for a specific Minecraft instance
    pub fn get_servers_for_instance(&self, instance_id: &str) -> Vec<&ServerInstance> {
        self.storage.get_servers_for_instance(instance_id)
    }

    /// Get Docker connections
    pub fn get_docker_connections(&self) -> Vec<&DockerConnection> {
        self.storage.get_docker_connections()
    }

    /// Get server logs
    pub async fn get_server_logs(&self, server_id: &str, lines: Option<u32>) -> Result<Vec<LogEntry>, String> {
        let server = self.storage.get_server(server_id)
            .ok_or("Server not found")?;

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            use bollard::container::LogsOptions;
            use futures::stream::StreamExt;

            let options = Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                tail: lines.unwrap_or(100).to_string(),
                timestamps: true,
                ..Default::default()
            });

            let mut logs_stream = docker.logs(container_id, options);
            let mut log_entries = Vec::new();

            while let Some(log) = logs_stream.next().await {
                match log {
                    Ok(output) => {
                        let log_line = output.to_string();
                        if let Some(parsed_log) = self.parse_log_line(&log_line) {
                            log_entries.push(parsed_log);
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to read logs: {}", e));
                    }
                }
            }

            Ok(log_entries)
        } else {
            Err("Server has no container".to_string())
        }
    }

    /// Parse a Docker log line into a LogEntry
    fn parse_log_line(&self, log_line: &str) -> Option<LogEntry> {
        // Docker log format with timestamps: "timestamp message"
        // We'll try to parse common Minecraft server log formats
        
        let cleaned_line = log_line.trim();
        if cleaned_line.is_empty() {
            return None;
        }

        // Try to extract timestamp (Docker includes it when timestamps=true)
        let (timestamp, message) = if let Some(space_pos) = cleaned_line.find(' ') {
            let potential_timestamp = &cleaned_line[..space_pos];
            // Check if it looks like an ISO timestamp
            if potential_timestamp.contains('T') && potential_timestamp.contains('Z') {
                (potential_timestamp.to_string(), cleaned_line[space_pos + 1..].to_string())
            } else {
                // No Docker timestamp, use current time
                (chrono::Utc::now().to_rfc3339(), cleaned_line.to_string())
            }
        } else {
            (chrono::Utc::now().to_rfc3339(), cleaned_line.to_string())
        };

        // Determine log level from message content
        let level = if message.contains("[ERROR]") || message.contains("ERROR") {
            LogLevel::Error
        } else if message.contains("[WARN]") || message.contains("WARN") || message.contains("WARNING") {
            LogLevel::Warn
        } else if message.contains("[DEBUG]") || message.contains("DEBUG") {
            LogLevel::Debug
        } else {
            LogLevel::Info
        };

        Some(LogEntry {
            timestamp,
            level,
            message: message.trim().to_string(),
        })
    }

    /// Execute RCON command in server container using mc-rcon
    pub async fn exec_command(&self, server_id: &str, minecraft_command: String) -> Result<String, String> {
        let server = self.storage.get_server(server_id)
            .ok_or("Server not found")?;

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
            use futures::stream::StreamExt;

            // Use rcon-cli command built into itzg's minecraft-server image
            let exec_options = CreateExecOptions {
                cmd: Some(vec!["rcon-cli".to_string(), minecraft_command]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let exec = docker.create_exec(container_id, exec_options)
                .await
                .map_err(|e| format!("Failed to create exec: {}", e))?;

            // Start exec
            let start_options = StartExecOptions { 
                detach: false,
                tty: false,
                output_capacity: None,
            };

            match docker.start_exec(&exec.id, Some(start_options)).await {
                Ok(StartExecResults::Attached { mut output, .. }) => {
                    let mut result = Vec::new();
                    
                    while let Some(chunk) = output.next().await {
                        match chunk {
                            Ok(output) => {
                                result.push(output.to_string());
                            }
                            Err(e) => {
                                return Err(format!("Failed to read exec output: {}", e));
                            }
                        }
                    }

                    let output = result.join("");
                    Ok(if output.trim().is_empty() { 
                        "Command executed successfully".to_string() 
                    } else { 
                        output 
                    })
                }
                Ok(StartExecResults::Detached) => {
                    Ok("Command executed in detached mode".to_string())
                }
                Err(e) => {
                    Err(format!("Failed to start exec: {}", e))
                }
            }
        } else {
            Err("Server has no container".to_string())
        }
    }

    /// Ensure itzg's Minecraft server image is available
    async fn ensure_itzg_image(&self, docker: &Docker, image_name: &str) -> Result<(), String> {
        // Check if image already exists locally
        let images = docker.list_images(Some(ListImagesOptions::<String> {
            filters: {
                let mut filters = HashMap::new();
                filters.insert("reference".to_string(), vec![image_name.to_string()]);
                filters
            },
            ..Default::default()
        }))
        .await
        .map_err(|e| format!("Failed to list images: {}", e))?;

        if !images.is_empty() {
            return Ok(()); // Image already exists
        }

        // Pull itzg's image if it doesn't exist
        println!("Pulling itzg/minecraft-server:latest image...");
        
        use bollard::image::CreateImageOptions;
        use futures::stream::StreamExt;
        
        let options = Some(CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        });

        let mut stream = docker.create_image(options, None, None);
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(status) = output.status {
                        println!("Docker pull: {}", status);
                    }
                },
                Err(e) => return Err(format!("Failed to pull image: {}", e)),
            }
        }

        println!("Successfully pulled itzg/minecraft-server:latest");
        Ok(())
    }
}