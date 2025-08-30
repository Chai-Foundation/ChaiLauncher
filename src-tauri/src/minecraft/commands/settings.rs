use tauri::command;
use crate::storage::StorageManager;

/// Get launcher settings
#[command]
pub async fn get_launcher_settings() -> Result<crate::storage::LauncherSettings, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    Ok(storage.get_settings().clone())
}

/// Update launcher settings
#[command]
pub async fn update_launcher_settings(settings: crate::storage::LauncherSettings) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let _ = storage.update_settings(settings).await;
    
    storage.save().await
        .map_err(|e| format!("Failed to save storage: {}", e))
}