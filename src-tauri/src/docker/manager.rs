use super::types::*;
use crate::minecraft::MinecraftInstance;
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
    servers: Vec<ServerInstance>,
}

impl DockerManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            servers: Vec::new(),
        }
    }

    /// Test connection to Docker
    pub async fn test_connection(&self, connection: &DockerConnection) -> Result<bool, String> {
        let docker = match connection.connection_type {
            DockerConnectionType::Local => {
                self.connect_local_docker()
                    .map_err(|e| format!("Failed to connect to local Docker: {}", e))?
            },
            DockerConnectionType::WindowsNamedPipe => {
                #[cfg(target_os = "windows")]
                {
                    Docker::connect_with_named_pipe_defaults()
                        .map_err(|e| format!("Failed to connect via Windows named pipe: {}", e))?
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err("Windows named pipe connection is only available on Windows".to_string());
                }
            },
            DockerConnectionType::UnixSocket => {
                #[cfg(not(target_os = "windows"))]
                {
                    Docker::connect_with_socket_defaults()
                        .map_err(|e| format!("Failed to connect via Unix socket: {}", e))?
                }
                #[cfg(target_os = "windows")]
                {
                    return Err("Unix socket connection is not available on Windows".to_string());
                }
            },
            DockerConnectionType::Remote => {
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to remote Docker: {}", e))?
            },
            DockerConnectionType::Swarm => {
                // For swarm, we connect to the manager node
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to Docker Swarm: {}", e))?
            }
        };

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
    pub async fn add_connection(&mut self, connection: DockerConnection) -> Result<(), String> {
        // Test connection first
        if !self.test_connection(&connection).await? {
            return Err("Failed to establish Docker connection".to_string());
        }

        let docker = match connection.connection_type {
            DockerConnectionType::Local => {
                self.connect_local_docker()
                    .map_err(|e| format!("Failed to connect to local Docker: {}", e))?
            },
            DockerConnectionType::WindowsNamedPipe => {
                #[cfg(target_os = "windows")]
                {
                    Docker::connect_with_named_pipe_defaults()
                        .map_err(|e| format!("Failed to connect via Windows named pipe: {}", e))?
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err("Windows named pipe connection is only available on Windows".to_string());
                }
            },
            DockerConnectionType::UnixSocket => {
                #[cfg(not(target_os = "windows"))]
                {
                    Docker::connect_with_socket_defaults()
                        .map_err(|e| format!("Failed to connect via Unix socket: {}", e))?
                }
                #[cfg(target_os = "windows")]
                {
                    return Err("Unix socket connection is not available on Windows".to_string());
                }
            },
            DockerConnectionType::Remote => {
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to remote Docker: {}", e))?
            },
            DockerConnectionType::Swarm => {
                let host = if let Some(port) = connection.port {
                    format!("tcp://{}:{}", connection.host, port)
                } else {
                    format!("tcp://{}:2376", connection.host)
                };
                Docker::connect_with_http(&host, 120, API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to Docker Swarm: {}", e))?
            }
        };

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

        // Create port bindings
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "25565/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(request.port.to_string()),
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
                ports.insert("25565/tcp".to_string(), HashMap::new());
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

        self.servers.push(server.clone());
        Ok(server)
    }

    /// Start a server
    pub async fn start_server(&mut self, server_id: &str) -> Result<(), String> {
        let server = self.servers.iter_mut()
            .find(|s| s.id == server_id)
            .ok_or("Server not found")?;

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            docker.start_container(container_id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| format!("Failed to start container: {}", e))?;

            server.status = ServerStatus::Starting;
            server.last_started = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// Stop a server
    pub async fn stop_server(&mut self, server_id: &str) -> Result<(), String> {
        let server = self.servers.iter_mut()
            .find(|s| s.id == server_id)
            .ok_or("Server not found")?;

        let docker = self.connections.get(&server.docker_connection_id)
            .ok_or("Docker connection not found")?;

        if let Some(container_id) = &server.container_id {
            let options = StopContainerOptions { t: 30 }; // 30 second timeout
            docker.stop_container(container_id, Some(options))
                .await
                .map_err(|e| format!("Failed to stop container: {}", e))?;

            server.status = ServerStatus::Stopping;
        }

        Ok(())
    }

    /// Get server status
    pub async fn get_server_status(&self, server_id: &str) -> Result<ServerStatus, String> {
        let server = self.servers.iter()
            .find(|s| s.id == server_id)
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
                match container.state.as_deref() {
                    Some("running") => Ok(ServerStatus::Running),
                    Some("exited") => Ok(ServerStatus::Stopped),
                    Some("created") => Ok(ServerStatus::Stopped),
                    Some("restarting") => Ok(ServerStatus::Starting),
                    _ => Ok(ServerStatus::Unknown),
                }
            } else {
                Ok(ServerStatus::Unknown)
            }
        } else {
            Ok(ServerStatus::Unknown)
        }
    }

    /// Remove a server
    pub async fn remove_server(&mut self, server_id: &str) -> Result<(), String> {
        let server_index = self.servers.iter().position(|s| s.id == server_id)
            .ok_or("Server not found")?;

        // Get the info we need before any async calls
        let server = self.servers[server_index].clone();
        let container_id = server.container_id.clone();
        let docker_connection_id = server.docker_connection_id.clone();

        // Handle Docker container cleanup if needed
        if let Some(container_id) = container_id {
            if let Some(docker) = self.connections.get(&docker_connection_id) {
                // Stop the server first without calling self.stop_server to avoid borrow issues
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

        // Remove from our list
        self.servers.remove(server_index);
        Ok(())
    }

    /// Get all servers
    pub fn get_servers(&self) -> &[ServerInstance] {
        &self.servers
    }

    /// Get servers for a specific Minecraft instance
    pub fn get_servers_for_instance(&self, instance_id: &str) -> Vec<&ServerInstance> {
        self.servers.iter()
            .filter(|s| s.minecraft_instance_id == instance_id)
            .collect()
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