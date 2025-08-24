use crate::mods::{ModManager, ModInfo, InstalledMod, ModLoader};
use crate::mods::api::ModApi;
use tauri::{command, AppHandle, Emitter};
use std::path::PathBuf;
use std::collections::HashMap;
use serde_json;

/// Search for mods across all available APIs
#[command]
pub async fn search_mods(
    query: String,
    game_version: Option<String>,
    mod_loader: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ModInfo>, String> {
    // For now, create a temporary mod manager to search
    // In a real implementation, this might use a global manager or cache
    let temp_instance_path = std::env::temp_dir().join("temp_mod_search");
    
    let manager = ModManager::new(temp_instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    let results = manager.search_mods(
        &query,
        game_version.as_deref(),
        mod_loader.as_deref(),
        limit.unwrap_or(20),
    ).await
    .map_err(|e| format!("Search failed: {}", e))?;
    
    Ok(results)
}

/// Get detailed information about a specific mod
#[command]
pub async fn get_mod_details(mod_id: String) -> Result<ModInfo, String> {
    let temp_instance_path = std::env::temp_dir().join("temp_mod_search");
    let manager = ModManager::new(temp_instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    // Try to get mod details from any available API
    for client in crate::mods::api::ApiClientFactory::create_all() {
        if let Ok(mod_info) = client.get_mod_details(&mod_id).await {
            return Ok(mod_info);
        }
    }
    
    Err(format!("Mod {} not found", mod_id))
}

/// Install a mod to a specific instance
#[command]
pub async fn install_mod(
    instance_id: String,
    mod_id: String,
    version_id: Option<String>,
    app_handle: AppHandle,
) -> Result<InstalledMod, String> {
    let instance_path = get_instance_path(&instance_id)?;
    let mut manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    let app_handle_clone = app_handle.clone();
    let mod_id_clone = mod_id.clone();
    let instance_id_clone = instance_id.clone();
    let installed_mod = manager.install_mod(
        &mod_id,
        version_id.as_deref(),
        move |downloaded, total| {
            let progress = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };
            
            let _ = app_handle_clone.emit("mod_install_progress", serde_json::json!({
                "instance_id": instance_id_clone,
                "mod_id": mod_id_clone,
                "progress": progress,
                "downloaded": downloaded,
                "total": total
            }));
        }
    ).await
    .map_err(|e| format!("Failed to install mod: {}", e))?;
    
    let _ = app_handle.emit("mod_installed", serde_json::json!({
        "instance_id": instance_id,
        "mod": installed_mod
    }));
    
    Ok(installed_mod)
}

/// Uninstall a mod from an instance
#[command]
pub async fn uninstall_mod(
    instance_id: String,
    mod_id: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let instance_path = get_instance_path(&instance_id)?;
    let mut manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    manager.uninstall_mod(&mod_id).await
        .map_err(|e| format!("Failed to uninstall mod: {}", e))?;
    
    let _ = app_handle.emit("mod_uninstalled", serde_json::json!({
        "instance_id": instance_id,
        "mod_id": mod_id
    }));
    
    Ok(())
}

/// Update a mod to the latest version
#[command]
pub async fn update_mod(
    instance_id: String,
    mod_id: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let instance_path = get_instance_path(&instance_id)?;
    let mut manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    let app_handle_clone = app_handle.clone();
    let mod_id_clone = mod_id.clone();
    let instance_id_clone = instance_id.clone();
    manager.update_mod(
        &mod_id,
        move |downloaded, total| {
            let progress = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };
            
            let _ = app_handle_clone.emit("mod_update_progress", serde_json::json!({
                "instance_id": instance_id_clone,
                "mod_id": mod_id_clone,
                "progress": progress,
                "downloaded": downloaded,
                "total": total
            }));
        }
    ).await
    .map_err(|e| format!("Failed to update mod: {}", e))?;
    
    let _ = app_handle.emit("mod_updated", serde_json::json!({
        "instance_id": instance_id,
        "mod_id": mod_id
    }));
    
    Ok(())
}

/// Get all installed mods for an instance
#[command]
pub async fn get_installed_mods(instance_id: String) -> Result<HashMap<String, InstalledMod>, String> {
    let instance_path = get_instance_path(&instance_id)?;
    let manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    Ok(manager.get_installed_mods().clone())
}

/// Enable or disable a mod
#[command]
pub async fn set_mod_enabled(
    instance_id: String,
    mod_id: String,
    enabled: bool,
    app_handle: AppHandle,
) -> Result<(), String> {
    let instance_path = get_instance_path(&instance_id)?;
    let mut manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    manager.set_mod_enabled(&mod_id, enabled).await
        .map_err(|e| format!("Failed to set mod enabled state: {}", e))?;
    
    let _ = app_handle.emit("mod_enabled_changed", serde_json::json!({
        "instance_id": instance_id,
        "mod_id": mod_id,
        "enabled": enabled
    }));
    
    Ok(())
}

/// Check for updates for all mods in an instance
#[command]
pub async fn check_mod_updates(
    instance_id: String,
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let instance_path = get_instance_path(&instance_id)?;
    let mut manager = ModManager::new(instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    let mods_with_updates = manager.check_all_updates().await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;
    
    let _ = app_handle.emit("mod_updates_checked", serde_json::json!({
        "instance_id": instance_id,
        "mods_with_updates": mods_with_updates
    }));
    
    Ok(mods_with_updates)
}

/// Get available mod loader versions for a minecraft version
#[command]
pub async fn get_mod_loader_versions(
    loader_name: String,
    mc_version: String,
) -> Result<Vec<String>, String> {
    let temp_instance_path = std::env::temp_dir().join("temp_loader_check");
    let loader_manager = crate::mods::loaders::ModLoaderManager::new(temp_instance_path);
    
    loader_manager.get_available_versions(&loader_name, &mc_version).await
        .map_err(|e| format!("Failed to get loader versions: {}", e))
}

/// Install a mod loader for an instance
#[command]
pub async fn install_mod_loader(
    instance_id: String,
    loader_name: String,
    loader_version: String,
    mc_version: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let instance_path = get_instance_path(&instance_id)?;
    let loader_manager = crate::mods::loaders::ModLoaderManager::new(instance_path);
    
    let loader = match loader_name.to_lowercase().as_str() {
        "forge" => ModLoader::Forge(loader_version.clone()),
        "fabric" => ModLoader::Fabric(loader_version.clone()),
        "quilt" => ModLoader::Quilt(loader_version.clone()),
        "neoforge" => ModLoader::NeoForge(loader_version.clone()),
        _ => return Err(format!("Unsupported loader: {}", loader_name)),
    };
    
    loader_manager.install_loader(&loader, &mc_version).await
        .map_err(|e| format!("Failed to install mod loader: {}", e))?;
    
    let _ = app_handle.emit("mod_loader_installed", serde_json::json!({
        "instance_id": instance_id,
        "loader": loader_name,
        "version": loader_version
    }));
    
    Ok(())
}

/// Get the installed mod loader for an instance
#[command]
pub async fn get_installed_mod_loader(instance_id: String) -> Result<Option<ModLoader>, String> {
    let instance_path = get_instance_path(&instance_id)?;
    let loader_manager = crate::mods::loaders::ModLoaderManager::new(instance_path);
    
    Ok(loader_manager.get_installed_loader().await)
}

/// Get featured/popular mods
#[command]
pub async fn get_featured_mods(
    game_version: Option<String>,
    mod_loader: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ModInfo>, String> {
    let temp_instance_path = std::env::temp_dir().join("temp_mod_search");
    let manager = ModManager::new(temp_instance_path).await
        .map_err(|e| format!("Failed to create mod manager: {}", e))?;
    
    let mut all_featured = Vec::new();
    
    for client in crate::mods::api::ApiClientFactory::create_all() {
        if let Ok(mut featured) = client.get_featured_mods(
            game_version.as_deref(),
            mod_loader.as_deref(),
            limit.unwrap_or(10),
        ).await {
            all_featured.append(&mut featured);
        }
    }
    
    // Remove duplicates and limit results
    all_featured.sort_by(|a, b| a.name.cmp(&b.name));
    all_featured.dedup_by(|a, b| a.name == b.name);
    
    if let Some(limit) = limit {
        all_featured.truncate(limit as usize);
    }
    
    Ok(all_featured)
}

/// Get mod categories
#[command]
pub async fn get_mod_categories() -> Result<Vec<String>, String> {
    let mut all_categories = Vec::new();
    
    for client in crate::mods::api::ApiClientFactory::create_all() {
        if let Ok(mut categories) = client.get_categories().await {
            all_categories.append(&mut categories);
        }
    }
    
    // Remove duplicates
    all_categories.sort();
    all_categories.dedup();
    
    Ok(all_categories)
}

// Helper function to get instance path
fn get_instance_path(instance_id: &str) -> Result<PathBuf, String> {
    // This should integrate with the existing instance management system
    // For now, we'll use a basic implementation
    let launcher_dir = crate::storage::get_launcher_dir();
    let instances_dir = launcher_dir.join("instances");
    let instance_path = instances_dir.join(instance_id);
    
    if !instance_path.exists() {
        return Err(format!("Instance {} not found", instance_id));
    }
    
    Ok(instance_path)
}