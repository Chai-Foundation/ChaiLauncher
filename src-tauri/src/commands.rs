use tauri::command;
use std::process::Command;
use crate::storage::StorageManager;

#[command]
pub async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[command]
pub async fn open_instance_folder(instance_id: String) -> Result<(), String> {
    let instance_id = instance_id;
    use crate::storage::StorageManager;
    
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    if let Some(instance) = storage.get_instance(&instance_id) {
        let path = instance.game_dir.to_string_lossy().to_string();
        open_folder(path).await
    } else {
        Err(format!("Instance not found: {}", instance_id))
    }
}

#[command]
pub async fn set_auth_token(token: String) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let mut settings = storage.get_settings().clone();
    settings.auth_token = Some(token);
    
    storage.update_settings(settings).await
        .map_err(|e| format!("Failed to save auth token: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_auth_token() -> Result<Option<String>, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    Ok(storage.get_settings().auth_token.clone())
}

#[command]
pub async fn clear_auth_token() -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let mut settings = storage.get_settings().clone();
    settings.auth_token = None;
    
    storage.update_settings(settings).await
        .map_err(|e| format!("Failed to clear auth token: {}", e))?;
    
    Ok(())
}