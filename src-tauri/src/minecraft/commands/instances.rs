use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{command, AppHandle, Emitter};
use tokio::fs;
use reqwest;
use serde_json;
use std::str::FromStr;
use lazy_static::lazy_static;

use crate::storage::{StorageManager, InstanceMetadata};
use crate::minecraft::{MinecraftInstance, AuthInfo};
use super::versions::{download_minecraft_assets_with_progress, download_minecraft_assets_with_progress_auto, download_version_manifest};

/// Create a new Minecraft instance with optional mod loader and modpack support
#[command]
pub async fn create_instance_with_loader(
    name: String,
    version: String,
    game_dir: String,
    mod_loader: Option<String>,
    modpack_id: Option<String>,
    modpack_version_id: Option<String>,
    modpack_platform: Option<String>,
    install_overrides: Option<bool>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let is_modpack = modpack_id.is_some();
    let is_modded = mod_loader.is_some() || is_modpack;
    let loader_name = mod_loader.clone().unwrap_or_else(|| if is_modpack { "fabric" } else { "vanilla" }.to_string());
    
    let instance = InstanceMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        version: version.clone(),
        modpack: modpack_id.clone(),
        modpack_version: modpack_version_id.clone(),
        game_dir: std::path::PathBuf::from(&game_dir),
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded,
        mods_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        size_mb: None,
        description: if is_modpack {
            Some(format!("Modpack instance: {} ({})", modpack_id.as_ref().unwrap(), modpack_platform.as_ref().unwrap_or(&"unknown".to_string())))
        } else if is_modded {
            Some(format!("Modded instance with {} for Minecraft {}", loader_name, version))
        } else {
            Some(format!("Vanilla Minecraft {} instance", version))
        },
        tags: if is_modpack {
            vec!["modpack".to_string(), modpack_platform.clone().unwrap_or_else(|| "unknown".to_string()), loader_name.clone()]
        } else if is_modded {
            vec!["modded".to_string(), loader_name.clone()]
        } else {
            vec!["vanilla".to_string()]
        },
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    // Save the instance first
    save_instance(instance.clone(), app_handle.clone()).await?;
    
    let instance_path = std::path::PathBuf::from(&game_dir);
    let mut final_version = version.clone();
    let mut mods_count = 0;
    let mut actual_minecraft_version = version.clone(); // For asset downloads
    
    // Handle modpack installation
    if let (Some(pack_id), Some(pack_version_id), Some(platform)) = (&modpack_id, &modpack_version_id, &modpack_platform) {
        println!("📦 Installing modpack '{}' from {} for instance '{}'...", pack_id, platform, name);
        
        // Use libium integration to install the modpack
        let mut modpack_manager = crate::libium_integration::modpacks::ModpackManager::new().await
            .map_err(|e| format!("Failed to initialize modpack manager: {}", e))?;
        
        let install_request = crate::libium_integration::types::InstallModpackRequest {
            instance_name: name.clone(),
            modpack_id: pack_id.clone(),
            version_id: pack_version_id.clone(),
            platform: platform.clone(),
            game_directory: Some(game_dir.clone()),
            install_overrides: install_overrides.unwrap_or(true),
        };
        
        // Create progress callback for modpack installation
        let app_handle_clone = app_handle.clone();
        let instance_id_clone = instance.id.clone();
        let progress_callback = move |current: u64, total: u64, message: String| {
            let _ = app_handle_clone.emit("modpack_install_progress", serde_json::json!({
                "instanceId": instance_id_clone,
                "current": current,
                "total": total,
                "message": message
            }));
        };
        
        modpack_manager.install_modpack(&install_request, app_handle.clone(), progress_callback).await
            .map_err(|e| format!("Failed to install modpack: {}", e))?;
        
        println!("✅ Modpack '{}' installed successfully", pack_id);
        
        // Get the actual Minecraft version from the modpack
        let (game_version, mod_loader_name) = modpack_manager.get_modpack_details(pack_id, pack_version_id, platform).await
            .unwrap_or_else(|_| (version.clone(), "fabric".to_string()));
        actual_minecraft_version = game_version;
        
        // Get the mod count from the mods directory
        let mods_dir = instance_path.join("mods");
        if mods_dir.exists() {
            if let Ok(entries) = tokio::fs::read_dir(&mods_dir).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_type) = entry.file_type().await {
                        if file_type.is_file() {
                            if let Some(extension) = entry.path().extension() {
                                if extension == "jar" {
                                    mods_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Don't install mod loader separately for modpacks since libium handles it
        
    } else if let Some(loader) = mod_loader {
        // Install mod loader for non-modpack instances
        println!("🔧 Installing {} mod loader for instance '{}'...", loader, name);
        
        // Create mods directory
        tokio::fs::create_dir_all(instance_path.join("mods")).await
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
        
        // Install the mod loader
        let mod_loader_profile = crate::libium_integration::install_mod_loader(&instance_path, &version, &loader).await
            .map_err(|e| format!("Failed to install mod loader: {}", e))?;
        
        // Update final version to use the mod loader profile
        final_version = mod_loader_profile;
        
        // Set up libium profile for mod management
        let mut manager = crate::libium_integration::LibiumManager::new().await
            .map_err(|e| format!("Failed to initialize libium: {}", e))?;
        
        manager.ensure_instance_profile(&instance.id, &instance_path).await
            .map_err(|e| format!("Failed to create instance profile: {}", e))?;
        
        let libium_loader = libium::config::structs::ModLoader::from_str(&loader)
            .map_err(|_| format!("Invalid mod loader: {}", loader))?;
        
        manager.update_profile_filters(&instance.id, vec![version.clone()], libium_loader).await
            .map_err(|e| format!("Failed to update profile filters: {}", e))?;
        
        println!("✅ {} mod loader installed successfully", loader);
    }
    
    // Update instance with final version and mod count
    if final_version != instance.version || mods_count > 0 {
        let mut updated_instance = instance.clone();
        updated_instance.version = final_version.clone();
        updated_instance.mods_count = mods_count;
        save_instance(updated_instance, app_handle.clone()).await?;
    }
    
    // Download assets for the new instance (use actual Minecraft version for assets)
    println!("📦 Downloading assets for new instance '{}'...", name);
    download_minecraft_assets_with_progress_auto(
        actual_minecraft_version,  // Use actual Minecraft version extracted from modpack or original version
        game_dir,
        &instance.id,
        &app_handle,
    ).await.map_err(|e| {
        println!("⚠️  Asset download failed: {}", e);
        format!("Instance created but asset download failed: {}", e)
    })?;
    
    println!("✅ Instance '{}' created successfully with assets", name);
    Ok(())
}

// Global map to track running instances by ID
lazy_static! {
    static ref RUNNING_INSTANCES: Arc<Mutex<HashMap<String, std::process::Child>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Watches a launched game so the running map does not keep reporting a
/// process that has already exited.
fn spawn_instance_monitor(app_handle: AppHandle, instance_id: String) {
    std::thread::spawn(move || loop {
        let finished = {
            let mut running = RUNNING_INSTANCES.lock().unwrap();
            match running.get_mut(&instance_id) {
                Some(child) => match child.try_wait() {
                    Ok(Some(_)) => {
                        running.remove(&instance_id);
                        Some(true)
                    }
                    Ok(None) => Some(false),
                    Err(e) => {
                        running.remove(&instance_id);
                        eprintln!("Failed to poll instance '{}': {}", instance_id, e);
                        Some(false)
                    }
                },
                None => None,
            }
        };

        match finished {
            None => break,
            Some(true) => {
                let _ = app_handle.emit("instances_updated", ());
                break;
            }
            Some(false) => std::thread::sleep(std::time::Duration::from_millis(400)),
        }
    });
}

/// Registers a freshly launched child process and starts its monitor.
fn track_launched_instance(
    app_handle: &AppHandle,
    instance_id: &str,
    child: std::process::Child,
) {
    RUNNING_INSTANCES
        .lock()
        .unwrap()
        .insert(instance_id.to_string(), child);
    spawn_instance_monitor(app_handle.clone(), instance_id.to_string());
}

/// Create a new Minecraft instance
#[command]
pub async fn create_instance(
    name: String,
    version: String,
    game_dir: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let instance = InstanceMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        version: version.clone(),
        modpack: None,
        modpack_version: None,
        game_dir: std::path::PathBuf::from(&game_dir),
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: false,
        mods_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        size_mb: None,
        description: None,
        tags: vec![],
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    // Save the instance first
    save_instance(instance.clone(), app_handle.clone()).await?;
    
    // Download assets for the new instance
    println!("📦 Downloading assets for new instance '{}'...", name);
    download_minecraft_assets_with_progress_auto(
        version,
        game_dir,
        &instance.id,
        &app_handle,
    ).await.map_err(|e| {
        println!("⚠️  Asset download failed: {}", e);
        format!("Instance created but asset download failed: {}", e)
    })?;
    
    println!("✅ Instance '{}' created successfully with assets", name);
    Ok(())
}

/// Launch Minecraft using the modular system
#[command]
pub async fn launch_minecraft(
    app_handle: AppHandle,
    instance: MinecraftInstance,
    java_path: Option<String>,
    memory: u32,
) -> Result<(), String> {
    // Use provided java_path or let the system choose
    let mut launch_instance = instance;
    if let Some(java) = java_path {
        launch_instance.java_path = Some(java);
    }
    
    let auth_info = get_auth_info().await.unwrap_or_default();
    println!("🔐 Using authentication: {} ({})", auth_info.username, auth_info.user_type);
    
    let instance_id = launch_instance.id.clone();

    match crate::minecraft::launch_minecraft(&launch_instance, Some(auth_info), memory).await {
        Ok(result) => {
            if result.success {
                println!("✓ Minecraft launched successfully with PID: {}", result.process_id);
                if let Some(child) = result.child {
                    track_launched_instance(&app_handle, &instance_id, child);
                }
                Ok(())
            } else {
                Err(result.error.unwrap_or("Unknown launch error".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Main launch command that matches the original signature
#[command]
pub async fn launch_instance(
    app_handle: AppHandle,
    instance_id: String,
    instance_path: String,
    version: String,
    java_path: String,
    memory: u32,
    jvm_args: Vec<String>,
) -> Result<(), String> {
    println!("🚀 Launching Minecraft {} using modular system", version);
    
    // Debug: Check Java requirements for version
    let java_version = crate::minecraft::versions::get_required_java_version(&version);
    println!("📋 Minecraft {} requires Java {}", version, java_version);
    
    // Try to get the actual instance metadata to determine if it's modded
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let (is_modded, modpack, modpack_version, mods_count) = if let Some(metadata) = storage.get_instance(&instance_id) {
        println!("📋 Found instance metadata: modded={}, modpack={:?}", metadata.is_modded, metadata.modpack);
        (metadata.is_modded, metadata.modpack.clone(), metadata.modpack_version.clone(), metadata.mods_count)
    } else {
        println!("⚠️  No instance metadata found for {}, assuming vanilla", instance_id);
        (false, None, None, 0)
    };
    
    // Create instance from parameters with correct modded status
    let instance = MinecraftInstance {
        id: instance_id,
        name: "temp".to_string(),
        version,
        modpack,
        modpack_version,
        game_dir: PathBuf::from(instance_path),
        java_path: Some(java_path),
        jvm_args: Some(jvm_args),
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded,
        mods_count,
        is_external: Some(false),
        external_launcher: None,
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    println!("🎮 Launching {} instance (modded: {})", if is_modded { "modded" } else { "vanilla" }, is_modded);
    
    // Try to get auth info from storage
    let auth_info = get_auth_info().await.unwrap_or_default();
    println!("🔐 Using authentication: {} ({})", auth_info.username, auth_info.user_type);
    
    // Launch using the modular system
    let instance_id = instance.id.clone();

    match crate::minecraft::launch_minecraft(&instance, Some(auth_info), memory).await {
        Ok(result) => {
            if result.success {
                println!("✓ Minecraft launched successfully with PID: {}", result.process_id);
                if let Some(child) = result.child {
                    track_launched_instance(&app_handle, &instance_id, child);
                }
                Ok(())
            } else {
                Err(result.error.unwrap_or("Unknown launch error".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Stop a running instance
#[command]
pub async fn stop_instance(instance_id: String) -> Result<(), String> {
    println!("🛑 Stopping instance: {}", instance_id);

    let child = RUNNING_INSTANCES.lock().unwrap().remove(&instance_id);

    match child {
        Some(mut child) => {
            let pid = child.id();
            child
                .kill()
                .map_err(|e| format!("Failed to stop instance (PID {}): {}", pid, e))?;
            let _ = child.wait();
            println!("✅ Instance '{}' stopped (PID: {})", instance_id, pid);
            Ok(())
        }
        None => {
            println!("⚠️  Instance '{}' is not tracked as running", instance_id);
            Ok(()) // Not an error if it's not running
        }
    }
}

/// Get list of currently running instance IDs
#[command]
pub async fn get_running_instances() -> Result<Vec<String>, String> {
    let running = RUNNING_INSTANCES.lock().unwrap();
    Ok(running.keys().cloned().collect())
}

/// Check if an instance is running
#[command]
pub async fn is_instance_running(instance_id: String) -> Result<bool, String> {
    let running = RUNNING_INSTANCES.lock().unwrap();
    Ok(running.contains_key(&instance_id))
}

/// Load instances from storage
#[command]
pub async fn load_instances() -> Result<Vec<MinecraftInstance>, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let instances: Vec<MinecraftInstance> = storage.get_all_instances()
        .into_iter()
        .cloned()
        .map(|metadata| metadata.into())
        .collect();
    
    Ok(instances)
}

/// Scan instances directory and import orphaned instances
#[command]
pub async fn import_orphaned_instances() -> Result<Vec<String>, String> {
    use std::fs;
    
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let instances_dir = storage.get_settings().instances_dir.clone();
    let mut imported = Vec::new();
    
    // Read instances directory
    let entries = fs::read_dir(&instances_dir)
        .map_err(|e| format!("Failed to read instances directory: {}", e))?;
        
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let instance_path = entry.path();
        
        if instance_path.is_dir() {
            let instance_name = instance_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
                
            // Check if this instance is already in config (try multiple possible IDs)
            let instance_id = format!("imported-{}", instance_name);
            let possible_ids = vec![
                instance_id.clone(),
                instance_name.clone(),
            ];
            
            let already_exists = possible_ids.iter().any(|id| storage.get_instance(id).is_some()) ||
                storage.get_all_instances().iter().any(|existing| existing.game_dir == instance_path);
                
            if already_exists {
                continue; // Already imported
            }
            
            // Look for version info to determine Minecraft version
            let versions_dir = instance_path.join("versions");
            if !versions_dir.exists() {
                continue; // Not a valid Minecraft instance
            }
            
            let mut minecraft_version = "unknown".to_string();
            if let Ok(version_entries) = fs::read_dir(&versions_dir) {
                for version_entry in version_entries {
                    if let Ok(version_entry) = version_entry {
                        let version_name = version_entry.file_name();
                        if let Some(version_str) = version_name.to_str() {
                            minecraft_version = version_str.to_string();
                            break; // Use first version found
                        }
                    }
                }
            }
            
            // Create instance metadata
            let metadata = InstanceMetadata {
                id: instance_id.clone(),
                name: instance_name.clone(),
                version: minecraft_version,
                modpack: None,
                modpack_version: None,
                game_dir: instance_path,
                java_path: None,
                jvm_args: None,
                last_played: None,
                total_play_time: 0,
                icon: None,
                is_modded: false,
                mods_count: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
                size_mb: None,
                description: Some(format!("Imported from existing instance directory")),
                tags: vec!["imported".to_string()],
                resolved_java_version: None,
                java_analysis_date: None,
            };
            
            // Add to storage
            storage.add_instance(metadata).await
                .map_err(|e| format!("Failed to save imported instance: {}", e))?;
                
            imported.push(instance_name.clone());
            println!("✅ Imported orphaned instance: {}", instance_name);
        }
    }
    
    Ok(imported)
}

/// Save instance to storage
#[command]
pub async fn save_instance(instance: InstanceMetadata, app_handle: AppHandle) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.add_instance(instance).await
        .map_err(|e| format!("Failed to save instance: {}", e))?;
    
    // No need to call save() again as add_instance() already calls it
    
    // Emit event to notify frontend that instances have been updated
    let _ = app_handle.emit("instances_updated", ());
    
    Ok(())
}

/// Delete instance
#[command]
pub async fn delete_instance(instance_id: String, app_handle: AppHandle) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.remove_instance(&instance_id).await
        .map_err(|e| format!("Failed to remove instance: {}", e))?;
    
    // No need to call save() again as remove_instance() already calls it
    
    // Emit event to notify frontend that instances have been updated
    let _ = app_handle.emit("instances_updated", ());
    
    Ok(())
}

/// Update instance
#[command]
pub async fn update_instance(instance: InstanceMetadata) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.update_instance(instance).await
        .map_err(|e| format!("Failed to update instance: {}", e))?;
    
    storage.save().await
        .map_err(|e| format!("Failed to save storage: {}", e))
}

/// Install Minecraft version
#[command]
pub async fn install_minecraft_version(
    version_id: String,
    instance_name: String,
    game_dir: String,
    instance_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let game_path = PathBuf::from(game_dir);
    let instance_dir = game_path.join(&instance_name);
    
    println!("🚀 Installing Minecraft {} for instance '{}'", version_id, instance_name);
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "starting",
        "progress": 0,
        "currentFile": "",
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    // Create instance directory structure
    fs::create_dir_all(&instance_dir).await
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    let versions_dir = instance_dir.join("versions");
    let libraries_dir = instance_dir.join("libraries");
    fs::create_dir_all(&versions_dir).await.map_err(|e| format!("Failed to create versions directory: {}", e))?;
    fs::create_dir_all(&libraries_dir).await.map_err(|e| format!("Failed to create libraries directory: {}", e))?;
    
    // Download version manifest
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "version",
        "progress": 5,
        "currentFile": "version manifest",
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    let version_manifest = download_version_manifest(&version_id, &instance_dir).await
        .map_err(|e| format!("Failed to download version manifest: {}", e))?;
    
    // Download version JAR
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "client",
        "progress": 15,
        "currentFile": format!("{}.jar", version_id),
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    println!("📥 Downloading Minecraft client JAR...");
    if let Some(downloads) = version_manifest.get("downloads") {
        if let Some(client) = downloads.get("client") {
            if let Some(url) = client.get("url").and_then(|u| u.as_str()) {
                let version_jar = versions_dir.join(&version_id).join(format!("{}.jar", version_id));
                fs::create_dir_all(version_jar.parent().unwrap()).await
                    .map_err(|e| format!("Failed to create version directory: {}", e))?;
                
                let response = reqwest::get(url).await
                    .map_err(|e| format!("Failed to download client JAR: {}", e))?;
                let bytes = response.bytes().await
                    .map_err(|e| format!("Failed to read client JAR: {}", e))?;
                fs::write(&version_jar, &bytes).await
                    .map_err(|e| format!("Failed to write client JAR: {}", e))?;
                
                println!("✓ Downloaded client JAR: {}", version_jar.display());
            }
        }
    }
    
    // Download libraries
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "libraries",
        "progress": 25,
        "currentFile": "libraries",
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    if let Some(libraries) = version_manifest.get("libraries").and_then(|v| v.as_array()) {
        println!("📦 Downloading {} libraries...", libraries.len());
        
        for (i, library) in libraries.iter().enumerate() {
            println!("🔄 Processing library {}/{}: {:?}", i + 1, libraries.len(), library.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"));
            
            if !should_include_library(library) {
                println!("⏭️  Skipping library {} (OS rules)", library.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"));
                continue;
            }
            
            if let Some(downloads) = library.get("downloads") {
                if let Some(artifact) = downloads.get("artifact") {
                    if let Some(url) = artifact.get("url").and_then(|u| u.as_str()) {
                        if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                            let lib_path = libraries_dir.join(path);
                            
                            if !lib_path.exists() {
                                println!("📥 Downloading artifact: {}", path);
                                if let Some(parent) = lib_path.parent() {
                                    fs::create_dir_all(parent).await
                                        .map_err(|e| format!("Failed to create library directory: {}", e))?;
                                }
                                
                                match tokio::time::timeout(
                                    std::time::Duration::from_secs(30),
                                    reqwest::get(url)
                                ).await {
                                    Ok(Ok(response)) => {
                                        match tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            response.bytes()
                                        ).await {
                                            Ok(Ok(bytes)) => {
                                                if let Err(e) = fs::write(&lib_path, &bytes).await {
                                                    println!("⚠️  Failed to write library {}: {}", path, e);
                                                } else {
                                                    println!("✅ Downloaded artifact: {}", path);
                                                }
                                            }
                                            Ok(Err(e)) => println!("⚠️  Failed to read library bytes {}: {}", path, e),
                                            Err(_) => println!("⚠️  Timeout reading library bytes: {}", path),
                                        }
                                    }
                                    Ok(Err(e)) => println!("⚠️  Failed to download library {}: {}", path, e),
                                    Err(_) => println!("⚠️  Timeout downloading library: {}", path),
                                }
                            }
                        }
                    }
                }
                
                // Handle natives for current OS
                let os_key = if cfg!(target_os = "windows") {
                    "natives-windows"
                } else if cfg!(target_os = "macos") {
                    "natives-osx"
                } else {
                    "natives-linux"
                };
                
                if let Some(classifiers) = downloads.get("classifiers") {
                    if let Some(native) = classifiers.get(os_key) {
                        if let Some(url) = native.get("url").and_then(|u| u.as_str()) {
                            if let Some(path) = native.get("path").and_then(|p| p.as_str()) {
                                let lib_path = libraries_dir.join(path);
                                
                                if !lib_path.exists() {
                                    println!("📥 Downloading native library: {}", path);
                                    if let Some(parent) = lib_path.parent() {
                                        fs::create_dir_all(parent).await
                                            .map_err(|e| format!("Failed to create library directory: {}", e))?;
                                    }
                                    
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(30),
                                        reqwest::get(url)
                                    ).await {
                                        Ok(Ok(response)) => {
                                            match tokio::time::timeout(
                                                std::time::Duration::from_secs(30),
                                                response.bytes()
                                            ).await {
                                                Ok(Ok(bytes)) => {
                                                    if let Err(e) = fs::write(&lib_path, &bytes).await {
                                                        println!("⚠️  Failed to write native library {}: {}", path, e);
                                                    } else {
                                                        println!("✅ Downloaded native library: {}", path);
                                                    }
                                                }
                                                Ok(Err(e)) => println!("⚠️  Failed to read native library bytes {}: {}", path, e),
                                                Err(_) => println!("⚠️  Timeout reading native library bytes: {}", path),
                                            }
                                        }
                                        Ok(Err(e)) => println!("⚠️  Failed to download native library {}: {}", path, e),
                                        Err(_) => println!("⚠️  Timeout downloading native library: {}", path),
                                    }
                                }
                            }
                        }
                    }
                }
            } else if let Some(name) = library.get("name").and_then(|n| n.as_str()) {
                // Handle libraries without downloads section (legacy format)
                let parts: Vec<&str> = name.split(':').collect();
                if parts.len() >= 3 {
                    let group = parts[0].replace('.', "/");
                    let artifact = parts[1];
                    let version = parts[2];
                    let jar_name = format!("{}-{}.jar", artifact, version);
                    let lib_path = libraries_dir.join(&group).join(artifact).join(version).join(&jar_name);
                    
                    if !lib_path.exists() {
                        println!("📥 Downloading legacy library: {}", jar_name);
                        if let Some(parent) = lib_path.parent() {
                            fs::create_dir_all(parent).await
                                .map_err(|e| format!("Failed to create library directory: {}", e))?;
                        }
                        
                        let mut downloaded = false;
                        
                        // Try Minecraft libraries repository first
                        let mc_url = format!("https://libraries.minecraft.net/{}/{}/{}/{}", 
                            group, artifact, version, jar_name);
                        
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            reqwest::get(&mc_url)
                        ).await {
                            Ok(Ok(response)) => {
                                if response.status().is_success() {
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(30),
                                        response.bytes()
                                    ).await {
                                        Ok(Ok(bytes)) => {
                                            if let Err(e) = fs::write(&lib_path, &bytes).await {
                                                println!("⚠️  Failed to write library {}: {}", jar_name, e);
                                            } else {
                                                println!("✅ Downloaded legacy library from MC repo: {}", jar_name);
                                                downloaded = true;
                                            }
                                        }
                                        Ok(Err(e)) => println!("⚠️  Failed to read library bytes {}: {}", jar_name, e),
                                        Err(_) => println!("⚠️  Timeout reading library bytes: {}", jar_name),
                                    }
                                }
                            }
                            Ok(Err(e)) => println!("⚠️  Failed to download library {}: {}", jar_name, e),
                            Err(_) => println!("⚠️  Timeout downloading library: {}", jar_name),
                        }
                        
                        // Fallback to Maven Central
                        if !downloaded {
                            let maven_url = format!(
                                "https://repo1.maven.org/maven2/{}/{}/{}/{}",
                                group, artifact, version, jar_name
                            );
                            
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                reqwest::get(&maven_url)
                            ).await {
                                Ok(Ok(response)) => {
                                    if response.status().is_success() {
                                        match tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            response.bytes()
                                        ).await {
                                            Ok(Ok(bytes)) => {
                                                if let Err(e) = fs::write(&lib_path, &bytes).await {
                                                    println!("⚠️  Failed to write library {}: {}", jar_name, e);
                                                } else {
                                                    println!("✅ Downloaded legacy library from Maven: {}", jar_name);
                                                }
                                            }
                                            Ok(Err(e)) => println!("⚠️  Failed to read library bytes from Maven {}: {}", jar_name, e),
                                            Err(_) => println!("⚠️  Timeout reading library bytes from Maven: {}", jar_name),
                                        }
                                    }
                                }
                                Ok(Err(e)) => println!("⚠️  Failed to download library from Maven {}: {}", jar_name, e),
                                Err(_) => println!("⚠️  Timeout downloading library from Maven: {}", jar_name),
                            }
                        }
                    }
                }
            }
            
            // Send live progress for every library
            let progress = 40 + ((i + 1) as f64 / libraries.len() as f64 * 30.0) as u32;
            let _ = app_handle.emit("install_progress", serde_json::json!({
                "instanceId": instance_id,
                "stage": "libraries",
                "progress": progress,
                "currentFile": format!("library_{}", i + 1),
                "bytesDownloaded": (i + 1) as u64,
                "totalBytes": libraries.len() as u64
            }));
            
            println!("🔄 Completed processing library {}/{}", i + 1, libraries.len());
            
            if (i + 1) % 10 == 0 {
                println!("📦 Downloaded {}/{} libraries", i + 1, libraries.len());
            }
        }
    }
    
    // Download assets
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "assets",
        "progress": 75,
        "currentFile": "assets",
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    println!("🎯 Starting asset download for version {}", version_id);
    download_minecraft_assets_with_progress(version_id.clone(), instance_dir.to_string_lossy().to_string(), &instance_id, &app_handle, &version_manifest).await?;
    println!("✅ Asset download completed for version {}", version_id);
    
    // Save the instance to storage so it persists
    let instance_metadata = InstanceMetadata {
        id: instance_id.clone(),
        name: instance_name.clone(),
        version: version_id.clone(),
        modpack: None,
        modpack_version: None,
        game_dir: instance_dir,
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: false,
        mods_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        size_mb: None,
        description: None,
        tags: vec![],
        resolved_java_version: None,
        java_analysis_date: None,
    };
    
    // Save to storage
    save_instance(instance_metadata, app_handle.clone()).await?;
    
    let _ = app_handle.emit("install_complete", serde_json::json!({
        "instanceId": instance_id,
        "success": true,
        "version": version_id
    }));
    
    println!("✅ Minecraft {} installation completed for '{}'", version_id, instance_name);
    Ok(())
}

/// Backup instance
#[command]
pub async fn backup_instance(instance_id: String, backup_path: String) -> Result<(), String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    if let Some(instance) = storage.get_instance(&instance_id) {
        let source_path = &instance.game_dir;
        let backup_dest = PathBuf::from(&backup_path);
        
        println!("📦 Creating backup of instance '{}'...", instance.name);
        
        // Create backup directory
        if let Some(parent) = backup_dest.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }
        
        // Copy instance directory to backup location
        copy_directory_recursive(source_path, &backup_dest).await
            .map_err(|e| format!("Failed to backup instance: {}", e))?;
        
        println!("✅ Instance '{}' backed up successfully", instance.name);
        Ok(())
    } else {
        Err(format!("Instance '{}' not found", instance_id))
    }
}

/// Restore instance
#[command]
pub async fn restore_instance(instance_id: String, backup_path: String) -> Result<(), String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    if let Some(instance) = storage.get_instance(&instance_id) {
        let backup_source = PathBuf::from(&backup_path);
        let restore_dest = &instance.game_dir;
        
        if !backup_source.exists() {
            return Err("Backup path does not exist".to_string());
        }
        
        println!("📦 Restoring instance '{}' from backup...", instance.name);
        
        // Remove existing instance directory
        if restore_dest.exists() {
            fs::remove_dir_all(restore_dest).await
                .map_err(|e| format!("Failed to remove existing instance: {}", e))?;
        }
        
        // Copy backup to instance location
        copy_directory_recursive(&backup_source, restore_dest).await
            .map_err(|e| format!("Failed to restore instance: {}", e))?;
        
        println!("✅ Instance '{}' restored successfully", instance.name);
        Ok(())
    } else {
        Err(format!("Instance '{}' not found", instance_id))
    }
}

/// Refresh instance sizes
#[command]
pub async fn refresh_instance_sizes() -> Result<(), String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    println!("📊 Refreshing instance sizes...");
    
    let instances = storage.get_all_instances();
    let mut updated_count = 0;
    
    for instance in instances {
        if instance.game_dir.exists() {
            let size = calculate_directory_size(&instance.game_dir).await?;
            // Note: StorageManager would need a method to update instance size
            // For now, just log the calculated size
            println!("Instance '{}': {} MB", instance.name, size / 1024 / 1024);
            updated_count += 1;
        }
    }
    
    println!("✅ Refreshed sizes for {} instances", updated_count);
    Ok(())
}


// Helper functions

/// Get authentication info for debugging (public version of get_auth_info)
#[command]
pub async fn get_auth_info_debug() -> Result<AuthInfo, String> {
    get_auth_info().await
}

/// Get authentication info from storage
async fn get_auth_info() -> Result<AuthInfo, String> {
    // First priority: Try to get Microsoft account info
    if let Ok(accounts) = crate::auth::get_stored_accounts().await {
        if let Some(account) = accounts.first() {
            // Check if token is still valid and refresh if needed
            match crate::auth::get_active_account_token().await {
                Ok(Some(active_token)) => {
                    return Ok(AuthInfo {
                        username: account.username.clone(),
                        uuid: account.uuid.clone(),
                        access_token: active_token,
                        user_type: "msa".to_string(),
                    });
                }
                Ok(None) => {
                    println!("⚠️  Microsoft account token expired or invalid");
                }
                Err(e) => {
                    println!("⚠️  Failed to get Microsoft account token: {}", e);
                }
            }
        }
    }
    
    // Second priority: Try to get manual auth token from settings
    let storage = crate::storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    if let Some(token) = storage.get_settings().auth_token.clone() {
        // If we have accounts but no valid token, use account info with manual token
        if let Ok(accounts) = crate::auth::get_stored_accounts().await {
            if let Some(account) = accounts.first() {
                return Ok(AuthInfo {
                    username: account.username.clone(),
                    uuid: account.uuid.clone(),
                    access_token: token,
                    user_type: "msa".to_string(),
                });
            }
        }
        
        // Require a Microsoft account when using manual tokens
        return Err("Manual authentication token provided, but no Microsoft account found. Please sign in with a Microsoft account first, then set your authentication token.".to_string());
    }
    
    // No valid authentication found - refuse to launch
    Err("No valid authentication found. ChaiLauncher requires either a Microsoft account or a valid authentication token. Please sign in with a Microsoft account or configure an authentication token.".to_string())
}

/// Check if a library should be included based on rules
fn should_include_library(library: &serde_json::Value) -> bool {
    let Some(rules) = library.get("rules").and_then(|v| v.as_array()) else {
        return true; // No rules means include
    };
    
    if rules.is_empty() {
        return true;
    }
    
    let current_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    };
    
    let mut allow = false;
    
    for rule in rules {
        if let Some(action) = rule.get("action").and_then(|v| v.as_str()) {
            let rule_applies = if let Some(os_obj) = rule.get("os") {
                if let Some(os_name) = os_obj.get("name").and_then(|v| v.as_str()) {
                    os_name == current_os
                } else {
                    true
                }
            } else {
                true
            };
            
            if rule_applies {
                match action {
                    "allow" => allow = true,
                    "disallow" => allow = false,
                    _ => {}
                }
            }
        }
    }
    
    allow
}

/// Copy directory recursively for backup/restore
async fn copy_directory_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    use walkdir::WalkDir;
    
    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let src_path = entry.path();
        let relative_path = src_path.strip_prefix(src)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        let dst_path = dst.join(relative_path);
        
        if src_path.is_dir() {
            fs::create_dir_all(&dst_path).await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).await
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            fs::copy(src_path, &dst_path).await
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    
    Ok(())
}

/// Calculate directory size recursively
async fn calculate_directory_size(dir: &PathBuf) -> Result<u64, String> {
    use walkdir::WalkDir;
    
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        if entry.path().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}

// Conversion implementation for InstanceMetadata -> MinecraftInstance
impl From<InstanceMetadata> for MinecraftInstance {
    fn from(metadata: InstanceMetadata) -> Self {
        Self {
            id: metadata.id,
            name: metadata.name,
            version: metadata.version,
            modpack: metadata.modpack,
            modpack_version: metadata.modpack_version,
            game_dir: metadata.game_dir,
            java_path: metadata.java_path,
            jvm_args: metadata.jvm_args,
            last_played: metadata.last_played,
            total_play_time: metadata.total_play_time,
            icon: metadata.icon,
            is_modded: metadata.is_modded,
            mods_count: metadata.mods_count,
            is_external: None,
            external_launcher: None,
            resolved_java_version: metadata.resolved_java_version,
            java_analysis_date: metadata.java_analysis_date,
        }
    }
}