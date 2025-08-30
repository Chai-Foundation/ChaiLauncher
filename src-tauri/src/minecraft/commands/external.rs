use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub launcher_type: String,
}

/// Detect external launcher instances
#[command]
pub async fn detect_all_external_instances() -> Result<Vec<ExternalInstance>, String> {
    // External launchers not supported - only our own instances
    Ok(vec![])
}

/// Detect GDLauncher instances specifically
#[command]
pub async fn detect_gdlauncher_instances() -> Result<Vec<ExternalInstance>, String> {
    // External launchers not supported - only our own instances  
    Ok(vec![])
}

/// Launch external instance
#[command]
pub async fn launch_external_instance(_instance_id: String, _instance_path: String) -> Result<(), String> {
    Err("External launcher support has been removed - use our own instances only".to_string())
}