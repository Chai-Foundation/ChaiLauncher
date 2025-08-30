use tauri::{command, Emitter};
use std::path::PathBuf;

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
    instance_dir: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let instance_path = PathBuf::from(&instance_dir).join(&instance_name);
    let installer = ModpackInstaller::new(instance_path.clone());

    // Get the specific version
    let versions = installer.get_modpack_versions(&project_id).await
        .map_err(|e| format!("Failed to get modpack versions: {}", e))?;
    
    let version = versions.into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| "Modpack version not found".to_string())?;

    // Install with progress reporting
    installer.download_and_install_modpack(&version, |progress| {
        let _ = app_handle.emit("modpack_install_progress", progress);
    }).await.map_err(|e| format!("Failed to install modpack: {}", e))?;

    println!("✅ Modpack '{}' installed successfully to: {}", version.name, instance_path.display());
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