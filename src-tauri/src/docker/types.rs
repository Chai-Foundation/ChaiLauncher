use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub connection_type: DockerConnectionType,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockerConnectionType {
    Local,
    WindowsNamedPipe,
    UnixSocket,
    Remote,
    Swarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub minecraft_instance_id: String,
    pub docker_connection_id: String,
    pub container_id: Option<String>,
    pub status: ServerStatus,
    pub port: u16,
    pub max_players: u32,
    pub memory_limit: u64, // in MB
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_started: Option<chrono::DateTime<chrono::Utc>>,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDeploymentRequest {
    pub name: String,
    pub minecraft_instance_id: String,
    pub docker_connection_id: String,
    pub port: u16,
    pub max_players: u32,
    pub memory_limit: u64,
    pub environment_vars: HashMap<String, String>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub network_in: u64,
    pub network_out: u64,
    pub uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: u64,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}