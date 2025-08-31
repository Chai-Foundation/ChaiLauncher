use crate::libium_integration::{LibiumManager, LibiumError, types::*};
use crate::storage;
use tauri::{command, AppHandle, Emitter};
use serde_json::json;
use libium::config::{
    structs::ModLoader,
    filters::{Filter, ReleaseChannel},
};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

/// Search for mods across all supported platforms  
#[command]
pub async fn search_mods(
    query: String,
    game_version: Option<String>,
    mod_loader: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ModSearchResult>, String> {
    let manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    let loader = if let Some(loader_str) = mod_loader {
        ModLoader::from_str(&loader_str).ok()
    } else {
        None
    };
    
    manager.search_mods(&query, game_version, loader, limit).await
        .map_err(|e| format!("Search failed: {}", e))
}

/// Get mod details using libium
#[command]
pub async fn get_mod_details(
    mod_id: String,
    platform: String,
) -> Result<ModSearchResult, String> {
    let manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Search for the specific mod to get details
    let results = manager.search_mods(&mod_id, None, None, Some(1)).await
        .map_err(|e| format!("Failed to get mod details: {}", e))?;
    
    results.into_iter()
        .find(|r| r.id == mod_id && r.project_type == platform)
        .ok_or_else(|| format!("Mod {} not found on {}", mod_id, platform))
}

/// Install a mod to an instance using libium
#[command]
pub async fn install_mod(
    instance_id: String,
    mod_id: String,
    platform: String,
    version_id: Option<String>,
    game_version: Option<String>,
    mod_loader: Option<String>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Get instance path
    let instance_path = get_instance_path(&instance_id)?;
    
    // Ensure profile exists for instance
    manager.ensure_instance_profile(&instance_id, &instance_path).await
        .map_err(|e| format!("Failed to create instance profile: {}", e))?;
    
    // Update profile with current game version and mod loader if provided
    if let (Some(game_ver), Some(loader_str)) = (game_version, mod_loader) {
        let loader = ModLoader::from_str(&loader_str)
            .map_err(|_| format!("Invalid mod loader: {}", loader_str))?;
        
        manager.update_profile_filters(&instance_id, vec![game_ver], loader).await
            .map_err(|e| format!("Failed to update profile filters: {}", e))?;
    }
    
    // Create filters for mod installation
    let filters = vec![]; // Use profile defaults
    
    // Emit progress start
    let _ = app_handle.emit("mod_install_started", json!({
        "instance_id": instance_id,
        "mod_id": mod_id,
        "platform": platform
    }));
    
    // Install the mod
    let mod_name = manager.add_mod_to_instance(&instance_id, &mod_id, &platform, filters).await
        .map_err(|e| format!("Failed to install mod: {}", e))?;
    
    // Emit completion event
    let _ = app_handle.emit("mod_install_completed", json!({
        "instance_id": instance_id,
        "mod_id": mod_id,
        "mod_name": mod_name,
        "platform": platform
    }));
    
    Ok(mod_name)
}

/// Uninstall a mod from an instance using libium
#[command]
pub async fn uninstall_mod(
    instance_id: String,
    mod_name: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    manager.remove_mod_from_instance(&instance_id, &mod_name).await
        .map_err(|e| format!("Failed to uninstall mod: {}", e))?;
    
    let _ = app_handle.emit("mod_uninstalled", json!({
        "instance_id": instance_id,
        "mod_name": mod_name
    }));
    
    Ok(())
}

/// Get all installed mods for an instance using libium
#[command]
pub async fn get_installed_mods(
    instance_id: String,
) -> Result<Vec<InstalledMod>, String> {
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Ensure profile exists
    let instance_path = get_instance_path(&instance_id)?;
    manager.ensure_instance_profile(&instance_id, &instance_path).await
        .map_err(|e| format!("Failed to create instance profile: {}", e))?;
    
    manager.get_instance_mods(&instance_id)
        .map_err(|e| format!("Failed to get installed mods: {}", e))
}

/// Scan mods in an instance directory using libium
#[command]
pub async fn scan_instance_mods(
    instance_id: String,
    app_handle: AppHandle,
) -> Result<Vec<ScannedMod>, String> {
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Ensure profile exists
    let instance_path = get_instance_path(&instance_id)?;
    manager.ensure_instance_profile(&instance_id, &instance_path).await
        .map_err(|e| format!("Failed to create instance profile: {}", e))?;
    
    let _ = app_handle.emit("mod_scan_started", json!({
        "instance_id": instance_id
    }));
    
    let scanned_mods = manager.scan_instance_mods(&instance_id).await
        .map_err(|e| format!("Failed to scan mods: {}", e))?;
    
    let _ = app_handle.emit("mod_scan_completed", json!({
        "instance_id": instance_id,
        "found_mods": scanned_mods.len()
    }));
    
    Ok(scanned_mods)
}

/// Search for modpacks using libium
#[command]
pub async fn search_modpacks(
    query: String,
    platform: String,
    game_version: Option<String>,
    mod_loader: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ModpackSearchResult>, String> {
    let modpack_manager = crate::libium_integration::modpacks::ModpackManager::new().await
        .map_err(|e| format!("Failed to initialize modpack manager: {}", e))?;
    
    modpack_manager.search_modpacks(
        &query,
        &platform,
        game_version,
        mod_loader,
        limit.unwrap_or(20),
        offset.unwrap_or(0),
    ).await
    .map_err(|e| format!("Search failed: {}", e))
}

/// Install a modpack using libium
#[command]
pub async fn install_modpack(
    request: InstallModpackRequest,
    app_handle: AppHandle,
) -> Result<String, String> {
    let mut modpack_manager = crate::libium_integration::modpacks::ModpackManager::new().await
        .map_err(|e| format!("Failed to initialize modpack manager: {}", e))?;
    
    let _ = app_handle.emit("modpack_install_started", json!({
        "modpack_id": request.modpack_id,
        "instance_name": request.instance_name,
        "platform": request.platform
    }));
    
    let app_handle_clone = app_handle.clone();
    let app_handle_clone2 = app_handle.clone();
    let instance_name_clone = request.instance_name.clone();
    let instance_name = modpack_manager.install_modpack(&request, app_handle_clone, move |current, total, stage| {
        let progress = if total > 0 { (current as f64 / total as f64 * 100.0) as u64 } else { 0 };
        
        let _ = app_handle_clone2.emit("modpack_install_progress", json!({
            "progress": progress,
            "stage": stage,
            "instance_name": instance_name_clone,
            "current": current,
            "total": total
        }));
    }).await
    .map_err(|e| format!("Failed to install modpack: {}", e))?;
    
    let _ = app_handle.emit("modpack_install_completed", json!({
        "modpack_id": request.modpack_id,
        "instance_name": &instance_name,
        "platform": request.platform
    }));
    
    Ok(instance_name)
}

/// Get modpack versions using libium
#[command]
pub async fn get_modpack_versions(
    modpack_id: String,
    platform: String,
) -> Result<Vec<ModpackVersion>, String> {
    let modpack_manager = crate::libium_integration::modpacks::ModpackManager::new().await
        .map_err(|e| format!("Failed to initialize modpack manager: {}", e))?;
    
    modpack_manager.get_modpack_versions(&modpack_id, &platform).await
        .map_err(|e| format!("Failed to get modpack versions: {}", e))
}

/// Update a modpack to a newer version
#[command]
pub async fn update_modpack(
    instance_name: String,
    new_version_id: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut modpack_manager = crate::libium_integration::modpacks::ModpackManager::new().await
        .map_err(|e| format!("Failed to initialize modpack manager: {}", e))?;
    
    let _ = app_handle.emit("modpack_update_started", json!({
        "instance_name": instance_name,
        "new_version_id": new_version_id
    }));
    
    let app_handle_clone = app_handle.clone();
    modpack_manager.update_modpack(&instance_name, &new_version_id, |current, total, stage| {
        let progress = if total > 0 { (current as f64 / total as f64 * 100.0) as u64 } else { 0 };
        
        let _ = app_handle.emit("modpack_update_progress", json!({
            "progress": progress,
            "stage": stage,
            "instance_name": instance_name,
            "current": current,
            "total": total
        }));
    }).await
    .map_err(|e| format!("Failed to update modpack: {}", e))?;
    
    let _ = app_handle.emit("modpack_update_completed", json!({
        "instance_name": instance_name,
        "new_version_id": new_version_id
    }));
    
    Ok(())
}

/// Update instance mod configuration
#[command]
pub async fn update_instance_mod_config(
    instance_id: String,
    game_versions: Vec<String>,
    mod_loader: String,
) -> Result<(), String> {
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    let loader = ModLoader::from_str(&mod_loader)
        .map_err(|_| format!("Invalid mod loader: {}", mod_loader))?;
    
    manager.update_profile_filters(&instance_id, game_versions, loader).await
        .map_err(|e| format!("Failed to update profile filters: {}", e))
}

/// Get featured/popular mods using libium
#[command]
pub async fn get_featured_mods(
    platform: Option<String>,
    game_version: Option<String>,
    mod_loader: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    category: Option<String>,
) -> Result<Vec<ModSearchResult>, String> {
    let manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Use empty query to get popular/featured mods
    let loader = if let Some(loader_str) = mod_loader {
        ModLoader::from_str(&loader_str).ok()
    } else {
        None
    };
    
    manager.search_mods("", game_version, loader, limit).await
        .map_err(|e| format!("Failed to get featured mods: {}", e))
}

/// Create a new instance with mod loader support
#[command]
pub async fn create_modded_instance(
    instance_name: String,
    game_version: String,
    mod_loader: Option<String>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "initializing",
        "progress": 0
    }));
    
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    // Create instance directory
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "creating_directories",
        "progress": 10
    }));
    
    let instances_dir = storage::get_launcher_dir().join("instances");
    let instance_path = instances_dir.join(&instance_name);
    
    tokio::fs::create_dir_all(&instance_path).await
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    // Create mods directory
    let mods_dir = instance_path.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await
        .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    // Determine mod loader
    let loader = if let Some(loader_str) = mod_loader {
        ModLoader::from_str(&loader_str)
            .map_err(|_| format!("Invalid mod loader: {}", loader_str))?
    } else {
        ModLoader::Fabric // Default to Fabric
    };
    
    // Install the mod loader first
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "installing_mod_loader",
        "progress": 30
    }));
    
    let loader_name = format!("{:?}", loader).to_lowercase();
    let mod_loader_profile = crate::libium_integration::install_mod_loader(&instance_path, &game_version, &loader_name).await
        .map_err(|e| format!("Failed to install mod loader: {}", e))?;
    
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "configuring_profile",
        "progress": 60
    }));
    
    // Create libium profile for this instance
    manager.ensure_instance_profile(&instance_name, &instance_path).await
        .map_err(|e| format!("Failed to create instance profile: {}", e))?;
    
    manager.update_profile_filters(&instance_name, vec![game_version.clone()], loader).await
        .map_err(|e| format!("Failed to update profile filters: {}", e))?;
    
    // Create instance metadata
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "registering_instance",
        "progress": 80
    }));
    
    let game_version_clone = game_version.clone();
    let instance_metadata = storage::InstanceMetadata {
        id: Uuid::new_v4().to_string(),
        name: instance_name.clone(),
        version: mod_loader_profile, // Use the mod loader profile as the version
        game_dir: instance_path,
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: true,
        mods_count: 0,
        modpack: None,
        modpack_version: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        size_mb: None,
        description: Some(format!("Modded instance with {}", format!("{:?}", loader))),
        tags: vec!["modded".to_string(), format!("{:?}", loader).to_lowercase()],
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    let mut storage_manager = storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage_manager.add_instance(instance_metadata).await
        .map_err(|e| format!("Failed to register instance: {}", e))?;
    
    let _ = app_handle.emit("instance_creation_progress", serde_json::json!({
        "instance_name": instance_name,
        "stage": "completed",
        "progress": 100
    }));
    
    let _ = app_handle.emit("instance_created", serde_json::json!({
        "instance_name": &instance_name,
        "mod_loader": format!("{:?}", loader),
        "game_version": game_version_clone
    }));
    
    Ok(instance_name)
}

/// Install mod loader for an existing instance
#[command]
pub async fn install_instance_mod_loader(
    instance_id: String,
    mod_loader: String,
    game_version: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let _ = app_handle.emit("mod_loader_install_started", json!({
        "instance_id": instance_id,
        "mod_loader": mod_loader,
        "game_version": game_version
    }));
    
    // Get instance path
    let instance_path = get_instance_path(&instance_id)?;
    
    // Install the mod loader
    let mod_loader_profile = crate::libium_integration::install_mod_loader(&instance_path, &game_version, &mod_loader).await
        .map_err(|e| format!("Failed to install mod loader: {}", e))?;
    
    // Update instance metadata to mark it as modded and use the mod loader profile
    let mut storage_manager = storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    if let Some(mut instance) = storage_manager.get_instance(&instance_id).cloned() {
        instance.is_modded = true;
        instance.version = mod_loader_profile.clone(); // Use the mod loader profile
        instance.tags.push("modded".to_string());
        instance.tags.push(mod_loader.clone());
        instance.description = Some(format!(
            "Instance with {} mod loader for Minecraft {}", 
            mod_loader, 
            game_version
        ));
        
        storage_manager.update_instance(instance).await
            .map_err(|e| format!("Failed to update instance metadata: {}", e))?;
    }
    
    // Create libium profile for mod management
    let mut manager = LibiumManager::new().await
        .map_err(|e| format!("Failed to initialize libium: {}", e))?;
    
    manager.ensure_instance_profile(&instance_id, &instance_path).await
        .map_err(|e| format!("Failed to create instance profile: {}", e))?;
    
    let loader = libium::config::structs::ModLoader::from_str(&mod_loader)
        .map_err(|_| format!("Invalid mod loader: {}", mod_loader))?;
    
    manager.update_profile_filters(&instance_id, vec![game_version.clone()], loader).await
        .map_err(|e| format!("Failed to update profile filters: {}", e))?;
    
    let _ = app_handle.emit("mod_loader_install_completed", json!({
        "instance_id": instance_id,
        "mod_loader": mod_loader,
        "game_version": game_version
    }));
    
    println!("✅ Successfully installed {} mod loader for instance {}", mod_loader, instance_id);
    Ok(())
}

/// Get available mod loaders
#[command]
pub async fn get_mod_loaders() -> Result<Vec<String>, String> {
    Ok(vec![
        "fabric".to_string(),
        "forge".to_string(),
        "quilt".to_string(),
        "neoforge".to_string(),
    ])
}

/// Diagnose and fix instances that may be incorrectly marked as vanilla
#[command]
pub async fn diagnose_instances() -> Result<Vec<String>, String> {
    let mut diagnostics = Vec::new();
    
    let storage_manager = storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let instances: Vec<_> = storage_manager.get_all_instances().iter().cloned().collect();
    
    for mut instance in instances {
        let instance_path = &instance.game_dir;
        let mods_dir = instance_path.join("mods");
        let versions_dir = instance_path.join("versions");
        
        // Check if mods directory exists and has mods
        let has_mods_dir = mods_dir.exists();
        let mods_count = if has_mods_dir {
            std::fs::read_dir(&mods_dir)
                .map(|entries| entries.filter_map(|e| e.ok()).filter(|e| {
                    e.path().extension().map_or(false, |ext| ext == "jar")
                }).count())
                .unwrap_or(0)
        } else {
            0
        };
        
        // Check for mod loader profiles
        let mut mod_loader_profiles = Vec::new();
        if versions_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&versions_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("fabric-loader") || name.contains("quilt-loader") || name.contains("forge") {
                        mod_loader_profiles.push(name);
                    }
                }
            }
        }
        
        // Generate diagnostic info
        let mut instance_diag = format!("Instance '{}' ({})", instance.name, instance.id);
        
        if instance.is_modded {
            instance_diag.push_str(" - Marked as MODDED");
        } else {
            instance_diag.push_str(" - Marked as VANILLA");
        }
        
        instance_diag.push_str(&format!(" - Version: {}", instance.version));
        instance_diag.push_str(&format!(" - Mods directory: {}", if has_mods_dir { "EXISTS" } else { "MISSING" }));
        instance_diag.push_str(&format!(" - Mods count: {} (metadata: {})", mods_count, instance.mods_count));
        
        if !mod_loader_profiles.is_empty() {
            instance_diag.push_str(&format!(" - Mod loader profiles found: {}", mod_loader_profiles.join(", ")));
        }
        
        // Check for issues
        if !instance.is_modded && (!mod_loader_profiles.is_empty() || mods_count > 0) {
            instance_diag.push_str(" - ⚠️  ISSUE: Should be marked as modded!");
        }
        
        if instance.is_modded && mod_loader_profiles.is_empty() {
            instance_diag.push_str(" - ⚠️  ISSUE: No mod loader profiles found!");
        }
        
        if instance.version.chars().all(|c| c.is_ascii_digit() || c == '.') && !mod_loader_profiles.is_empty() {
            instance_diag.push_str(" - ⚠️  ISSUE: Using vanilla version but mod loader installed!");
        }
        
        diagnostics.push(instance_diag);
    }
    
    Ok(diagnostics)
}

/// Fix instances that are incorrectly marked as vanilla
#[command]
pub async fn fix_vanilla_instances() -> Result<Vec<String>, String> {
    let mut fixed_instances = Vec::new();
    
    let mut storage_manager = storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;

    // Get all instances by value to avoid borrow conflicts
    let cloned_manager = storage_manager.clone();
    let instances_vec = cloned_manager.get_all_instances().into_iter().collect::<Vec<_>>();

    for instance in instances_vec {
        let instance_path = &instance.game_dir;
        let mods_dir = instance_path.join("mods");
        let versions_dir = instance_path.join("versions");
        
        // Check if mods directory exists and has mods
        let mods_count = if mods_dir.exists() {
            std::fs::read_dir(&mods_dir)
                .map(|entries| entries.filter_map(|e| e.ok()).filter(|e| {
                    e.path().extension().map_or(false, |ext| ext == "jar")
                }).count())
                .unwrap_or(0)
        } else {
            0
        };
        
        // Check for mod loader profiles
        let mut mod_loader_profile = None;
        if versions_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&versions_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("fabric-loader") || name.contains("quilt-loader") || name.contains("forge") {
                        mod_loader_profile = Some(name);
                        break;
                    }
                }
            }
        }
        
        let mut needs_update = false;
        let mut changes = Vec::new();
        let mut updated_instance = instance.clone();
        
        // Fix modded status
        if !updated_instance.is_modded && (mod_loader_profile.is_some() || mods_count > 0) {
            updated_instance.is_modded = true;
            needs_update = true;
            changes.push("marked as modded".to_string());
        }
        
        // Fix mod count
        if updated_instance.mods_count != mods_count as u32 {
            updated_instance.mods_count = mods_count as u32;
            needs_update = true;
            changes.push(format!("updated mod count to {}", mods_count));
        }
        
        // Fix version profile
        if let Some(ref profile) = mod_loader_profile {
            if updated_instance.version != *profile {
                updated_instance.version = profile.clone();
                needs_update = true;
                changes.push(format!("updated version to {}", profile));
            }
        }
        
        // Update tags
        if updated_instance.is_modded && !updated_instance.tags.contains(&"modded".to_string()) {
            updated_instance.tags.push("modded".to_string());
            needs_update = true;
            changes.push("added modded tag".to_string());
        }
        
        if needs_update {
            storage_manager.update_instance(updated_instance.clone()).await
                .map_err(|e| format!("Failed to update instance {}: {}", updated_instance.name, e))?;
            
            fixed_instances.push(format!("Fixed '{}': {}", updated_instance.name, changes.join(", ")));
        }
    }
    
    Ok(fixed_instances)
}

/// Get mod categories from all platforms
#[command]
pub async fn get_mod_categories() -> Result<Vec<String>, String> {
    // This would collect categories from all supported platforms
    // For now, return common Minecraft mod categories
    Ok(vec![
        "adventure".to_string(),
        "cursed".to_string(),
        "decoration".to_string(),
        "economy".to_string(),
        "equipment".to_string(),
        "food".to_string(),
        "library".to_string(),
        "magic".to_string(),
        "management".to_string(),
        "minigame".to_string(),
        "misc".to_string(),
        "optimization".to_string(),
        "social".to_string(),
        "storage".to_string(),
        "technology".to_string(),
        "transportation".to_string(),
        "utility".to_string(),
        "worldgen".to_string(),
    ])
}

// Helper function to get instance path
fn get_instance_path(instance_id: &str) -> Result<PathBuf, String> {
    let instances_dir = storage::get_launcher_dir().join("instances");
    let instance_path = instances_dir.join(instance_id);
    
    if !instance_path.exists() {
        return Err(format!("Instance {} not found", instance_id));
    }
    
    Ok(instance_path)
}