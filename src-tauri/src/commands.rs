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
    
    println!("🔍 Looking for instance with ID: {}", instance_id);
    
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    // Debug: List all available instances
    println!("📋 Available instances:");
    let all_instances = storage.get_all_instances();
    for inst in &all_instances {
        println!("  - ID: '{}', Name: '{}'", inst.id, inst.name);
    }
    
    if let Some(instance) = storage.get_instance(&instance_id) {
        let path = instance.game_dir.to_string_lossy().to_string();
        
        // Check if the directory exists before trying to open it
        if !std::path::Path::new(&path).exists() {
            // Create the directory if it doesn't exist
            println!("Creating missing instance directory: {}", path);
            tokio::fs::create_dir_all(&path).await
                .map_err(|e| format!("Failed to create instance directory: {}", e))?;
        }
        
        println!("Opening instance folder: {}", path);
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

/// Get current authentication status for debugging
#[command]
pub async fn get_auth_status() -> Result<serde_json::Value, String> {
    let mut status = serde_json::Map::new();
    
    // Check Microsoft accounts
    match crate::auth::get_stored_accounts().await {
        Ok(accounts) => {
            status.insert("microsoft_accounts_count".to_string(), serde_json::Value::from(accounts.len()));
            if let Some(account) = accounts.first() {
                status.insert("microsoft_username".to_string(), serde_json::Value::from(account.username.clone()));
                status.insert("microsoft_uuid".to_string(), serde_json::Value::from(account.uuid.clone()));
                
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let expires_in = account.expires_at.saturating_sub(current_time);
                status.insert("microsoft_token_expires_in".to_string(), serde_json::Value::from(expires_in));
                status.insert("microsoft_token_valid".to_string(), serde_json::Value::from(expires_in > 300));
            }
        }
        Err(e) => {
            status.insert("microsoft_accounts_error".to_string(), serde_json::Value::from(e));
        }
    }
    
    // Check manual auth token
    match get_auth_token().await {
        Ok(token) => {
            status.insert("manual_token_set".to_string(), serde_json::Value::from(token.is_some()));
            if let Some(token) = token {
                status.insert("manual_token_length".to_string(), serde_json::Value::from(token.len()));
            }
        }
        Err(e) => {
            status.insert("manual_token_error".to_string(), serde_json::Value::from(e));
        }
    }
    
    // Check what auth would be used for launch
    match crate::minecraft::commands::get_auth_info_debug().await {
        Ok(auth_info) => {
            status.insert("active_username".to_string(), serde_json::Value::from(auth_info.username));
            status.insert("active_uuid".to_string(), serde_json::Value::from(auth_info.uuid));
            status.insert("active_user_type".to_string(), serde_json::Value::from(auth_info.user_type));
            status.insert("active_token_length".to_string(), serde_json::Value::from(auth_info.access_token.len()));
        }
        Err(e) => {
            status.insert("active_auth_error".to_string(), serde_json::Value::from(e));
        }
    }
    
    Ok(serde_json::Value::Object(status))
}