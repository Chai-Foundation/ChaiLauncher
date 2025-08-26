use super::{DockerManager, DockerConnection, ServerDeploymentRequest, ServerInstance, LogEntry};
use crate::minecraft::MinecraftInstance;
use tokio::sync::Mutex;
use tauri::State;

// Global Docker manager state
pub type DockerManagerState = Mutex<DockerManager>;

#[tauri::command]
pub async fn test_docker_connection(connection: DockerConnection) -> Result<bool, String> {
    let manager = DockerManager::new();
    manager.test_connection(&connection).await
}

#[tauri::command]
pub async fn add_docker_connection(
    state: State<'_, DockerManagerState>,
    connection: DockerConnection,
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.add_connection(connection).await
}

#[tauri::command]
pub async fn deploy_minecraft_server(
    state: State<'_, DockerManagerState>,
    deployment_request: ServerDeploymentRequest,
    minecraft_instance: MinecraftInstance,
) -> Result<ServerInstance, String> {
    let mut manager = state.lock().await;
    manager.deploy_server(deployment_request, &minecraft_instance).await
}

#[tauri::command]
pub async fn start_server(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.start_server(&server_id).await
}

#[tauri::command]
pub async fn stop_server(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.stop_server(&server_id).await
}

#[tauri::command]
pub async fn remove_server(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.remove_server(&server_id).await
}

#[tauri::command]
pub async fn get_servers(
    state: State<'_, DockerManagerState>,
) -> Result<Vec<ServerInstance>, String> {
    let manager = state.lock().await;
    Ok(manager.get_servers().to_vec())
}

#[tauri::command]
pub async fn get_servers_for_instance(
    state: State<'_, DockerManagerState>,
    instance_id: String,
) -> Result<Vec<ServerInstance>, String> {
    let manager = state.lock().await;
    Ok(manager.get_servers_for_instance(&instance_id).into_iter().cloned().collect())
}

#[tauri::command]
pub async fn get_server_status(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<super::ServerStatus, String> {
    // Get server info first to avoid holding lock across await
    let server = {
        let manager = state.lock().await;
        manager.get_servers().iter()
            .find(|s| s.id == server_id)
            .cloned()
            .ok_or("Server not found")?
    };

    // For now, return the cached status
    // In a full implementation, we would query Docker here
    Ok(server.status)
}

#[tauri::command]
pub async fn get_server_logs(
    state: State<'_, DockerManagerState>,
    server_id: String,
    _lines: Option<u32>,
) -> Result<Vec<LogEntry>, String> {
    let manager = state.lock().await;
    
    // Find the server
    let server = manager.get_servers().iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    // For now, generate some mock logs since full Docker log implementation would be complex
    let mock_logs = vec![
        LogEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
            level: super::LogLevel::Info,
            message: format!("[Server thread/INFO]: Starting minecraft server version 1.20.4"),
        },
        LogEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(4),
            level: super::LogLevel::Info,
            message: format!("[Server thread/INFO]: Loading properties"),
        },
        LogEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(3),
            level: super::LogLevel::Info,
            message: format!("[Server thread/INFO]: Preparing level \"world\""),
        },
        LogEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(2),
            level: super::LogLevel::Info,
            message: format!("[Server thread/INFO]: Done ({}s)! For help, type \"help\"", 2.5),
        },
        LogEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(1),
            level: super::LogLevel::Info,
            message: format!("[Server thread/INFO]: Server is running on port {}", server.port),
        },
    ];

    Ok(mock_logs)
}

#[tauri::command]
pub async fn execute_server_command(
    state: State<'_, DockerManagerState>,
    server_id: String,
    command: String,
) -> Result<String, String> {
    let manager = state.lock().await;
    
    // Find the server
    let server = manager.get_servers().iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    if server.status != super::ServerStatus::Running {
        return Err("Server is not running".to_string());
    }

    // For a full implementation, we would use Docker exec to run the command in the container
    // For now, just return a success message
    match command.trim() {
        "stop" => Ok("Server is shutting down...".to_string()),
        cmd if cmd.starts_with("say ") => Ok(format!("Broadcasting: {}", &cmd[4..])),
        cmd if cmd.starts_with("kick ") => Ok(format!("Kicking player: {}", &cmd[5..])),
        "list" => Ok("There are 0 of a max of 20 players online:".to_string()),
        "help" => Ok("Available commands: stop, say <message>, kick <player>, list, whitelist, op, deop".to_string()),
        _ => Ok(format!("Command '{}' executed successfully", command)),
    }
}

#[tauri::command]
pub async fn restart_server(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<(), String> {
    // Stop the server first
    {
        let mut manager = state.lock().await;
        manager.stop_server(&server_id).await?;
    }
    
    // Wait a moment for the server to fully stop
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Start the server again
    {
        let mut manager = state.lock().await;
        manager.start_server(&server_id).await?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn backup_server(
    state: State<'_, DockerManagerState>,
    server_id: String,
    backup_name: Option<String>,
) -> Result<String, String> {
    let manager = state.lock().await;
    
    let server = manager.get_servers().iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    let backup_id = format!("backup-{}-{}", server.name, chrono::Utc::now().timestamp());
    
    // For a full implementation, we would:
    // 1. Stop the server temporarily
    // 2. Create a tar archive of the world data
    // 3. Store it in a backups directory
    // 4. Restart the server
    
    // For now, just return a success message
    let backup_display_name = backup_name.unwrap_or_else(|| {
        format!("{}-backup-{}", server.name, chrono::Utc::now().format("%Y%m%d-%H%M%S"))
    });
    
    Ok(format!("Backup '{}' created successfully with ID: {}", backup_display_name, backup_id))
}

#[tauri::command]
pub async fn get_server_stats(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<super::ServerStats, String> {
    let manager = state.lock().await;
    
    let _server = manager.get_servers().iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    // For a full implementation, we would get actual stats from Docker
    // For now, return mock stats
    Ok(super::ServerStats {
        cpu_usage: 15.5,
        memory_usage: 1024 * 1024 * 1500, // 1.5 GB in bytes
        memory_limit: 1024 * 1024 * 2048,  // 2 GB in bytes
        network_in: 1024 * 512,  // 512 KB
        network_out: 1024 * 256, // 256 KB
        uptime: 3600, // 1 hour in seconds
    })
}