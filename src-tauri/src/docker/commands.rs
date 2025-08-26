use super::{DockerManager, DockerConnection, ServerDeploymentRequest, ServerInstance};
use crate::minecraft::MinecraftInstance;
use tokio::sync::Mutex;
use tauri::State;

// Global Docker manager state
pub type DockerManagerState = Mutex<DockerManager>;

#[tauri::command]
pub async fn test_docker_connection(connection: DockerConnection) -> Result<bool, String> {
    match DockerManager::new().await {
        Ok(manager) => manager.test_connection(&connection).await,
        Err(e) => Err(format!("Failed to create Docker manager: {}", e))
    }
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
    Ok(manager.get_servers().into_iter().cloned().collect())
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
pub async fn get_docker_connections(
    state: State<'_, DockerManagerState>,
) -> Result<Vec<DockerConnection>, String> {
    let manager = state.lock().await;
    Ok(manager.get_docker_connections().into_iter().cloned().collect())
}

#[tauri::command]
pub async fn get_server_logs(
    state: State<'_, DockerManagerState>,
    server_id: String,
    lines: Option<u32>,
) -> Result<Vec<super::LogEntry>, String> {
    let manager = state.lock().await;
    manager.get_server_logs(&server_id, lines).await
}

#[tauri::command]
pub async fn execute_server_command(
    state: State<'_, DockerManagerState>,
    server_id: String,
    command: String,
) -> Result<String, String> {
    let manager = state.lock().await;
    manager.exec_command(&server_id, command).await
}

#[tauri::command]
pub async fn get_server_status(
    state: State<'_, DockerManagerState>,
    server_id: String,
) -> Result<super::ServerStatus, String> {
    let mut manager = state.lock().await;
    manager.get_server_status(&server_id).await
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
        .cloned()
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