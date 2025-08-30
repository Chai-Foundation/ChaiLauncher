use tauri::{command, Emitter};
use serde::Deserialize;
use std::path::PathBuf;
use uuid;
use chrono;

use super::types::*;

/// Search for modpacks
#[command]
pub async fn search_modpacks(query: String, platform: String, limit: u32, offset: Option<u32>) -> Result<Vec<ModrinthPack>, String> {
    let offset = offset.unwrap_or(0);
    
    match platform.as_str() {
        "modrinth" => {
            let temp_dir = std::env::temp_dir().join("temp_search");
            let installer = ModpackInstaller::new(temp_dir);
            
            installer.search_modrinth_packs(&query, limit, offset).await
                .map_err(|e| e.to_string())
        },
        _ => Err("Unsupported platform".to_string())
    }
}

/// Get modpack versions
#[command]
pub async fn get_modpack_versions(
    project_id: String,
    platform: String,
) -> Result<Vec<ModrinthVersion>, String> {
    match platform.as_str() {
        "modrinth" => {
            let temp_dir = std::env::temp_dir().join("temp_versions");
            let installer = ModpackInstaller::new(temp_dir);
            
            installer.get_modpack_versions(&project_id).await
                .map_err(|e| e.to_string())
        },
        _ => Err("Unsupported platform".to_string())
    }
}

/// Install a modpack
#[command]
pub async fn install_modpack(
    project_id: String,
    version_id: String,
    instance_name: String,
    game_dir: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Create the instance directory path like other commands
    let game_path = PathBuf::from(&game_dir);
    let instance_path = game_path.join(&instance_name);
    let installer = ModpackInstaller::new(instance_path.clone());

    println!("🚀 Starting modpack installation to: {}", instance_path.display());
    
    // Get the specific version
    let versions = installer.get_modpack_versions(&project_id).await
        .map_err(|e| format!("Failed to get modpack versions: {}", e))?;
    
    let version = versions.into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| "Modpack version not found".to_string())?;

    println!("📦 Found modpack version: {} ({})", version.name, version.version_number);

    // Install with progress reporting - use direct .mrpack parsing for Modrinth packs
    if version.files.is_empty() {
        // Fallback to old method for non-Modrinth or legacy packs
        installer.download_and_install_modpack(&version, |progress| {
            let _ = app_handle.emit("modpack_install_progress", progress);
        }).await.map_err(|e| format!("Failed to install modpack: {}", e))?;
    } else {
        // Use new method for proper .mrpack handling with individual mod downloads
        installer.install_mrpack_direct(&version, |progress| {
            let _ = app_handle.emit("modpack_install_progress", progress);
        }).await.map_err(|e| format!("Failed to install modpack: {}", e))?;
    }

    // Register the instance in ChaiLauncher's storage system like a real launcher
    println!("📝 Registering instance '{}' in ChaiLauncher storage", instance_name);
    
    // Determine Minecraft version from modpack metadata
    let minecraft_version = version.game_versions.first()
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string()); // Default fallback
    
    // Create instance metadata
    let instance_metadata = crate::storage::InstanceMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        name: instance_name.clone(),
        version: minecraft_version,
        game_dir: instance_path.clone(),
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: true,
        mods_count: version.files.len() as u32,
        modpack: Some(version.name.clone()),
        modpack_version: Some(version.version_number.clone()),
        created_at: chrono::Utc::now().to_rfc3339(),
        size_mb: None,
        description: None,
        tags: vec![],
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    // Add to storage
    let mut storage = crate::storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.add_instance(instance_metadata).await
        .map_err(|e| format!("Failed to register instance: {}", e))?;
    
    println!("✅ Modpack '{}' installed and registered successfully to: {}", version.name, instance_path.display());
    Ok(())
}

/// Create a modpack from an existing instance
#[command]
pub async fn create_modpack(
    request: ModpackCreationRequest,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let creator = ModpackCreator::new();
    let instance_id = request.instance_id.clone(); // Clone before moving into closure
    
    let app_handle_clone = app_handle.clone();
    let result = creator.create_modpack(&request, move |progress, stage| {
        let _ = app_handle_clone.emit("modpack_creation_progress", ModpackCreationProgress {
            instance_id: instance_id.clone(),
            progress,
            stage,
        });
    }).await;
    
    match result {
        Ok(modpack_path) => Ok(modpack_path),
        Err(e) => Err(format!("Failed to create modpack: {}", e))
    }
}