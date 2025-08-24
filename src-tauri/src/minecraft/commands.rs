use super::*;
use crate::storage::{StorageManager, InstanceMetadata};
use crate::minecraft::versions::load_version_manifest;
use tauri::{command, Emitter, AppHandle};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use reqwest;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub launcher_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<MinecraftVersionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

/// Get Minecraft versions from Mojang API
#[command]
pub async fn get_minecraft_versions() -> Result<VersionManifest, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    
    let response = reqwest::get(url).await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    
    let manifest: VersionManifest = response.json().await
        .map_err(|e| format!("Failed to parse version manifest: {}", e))?;
    
    Ok(manifest)
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
    };
    
    // Save the instance first
    save_instance(instance.clone(), app_handle.clone()).await?;
    
    // Download assets for the new instance
    println!("📦 Downloading assets for new instance '{}'...", name);
    download_minecraft_assets_with_progress(
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
    
    match crate::minecraft::launch_minecraft(&launch_instance, Some(auth_info), memory).await {
        Ok(result) => {
            if result.success {
                println!("✓ Minecraft launched successfully with PID: {}", result.process_id);
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
    
    // Create instance from parameters
    let instance = MinecraftInstance {
        id: instance_id,
        name: "temp".to_string(),
        version,
        modpack: None,
        modpack_version: None,
        game_dir: PathBuf::from(instance_path),
        java_path: Some(java_path),
        jvm_args: Some(jvm_args),
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: false,
        mods_count: 0,
        is_external: Some(false),
        external_launcher: None,
    };
    
    // Try to get auth info from storage
    let auth_info = get_auth_info().await.unwrap_or_default();
    println!("🔐 Using authentication: {} ({})", auth_info.username, auth_info.user_type);
    
    // Launch using the modular system
    match crate::minecraft::launch_minecraft(&instance, Some(auth_info), memory).await {
        Ok(result) => {
            if result.success {
                println!("✓ Minecraft launched successfully with PID: {}", result.process_id);
                Ok(())
            } else {
                Err(result.error.unwrap_or("Unknown launch error".to_string()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Get bundled Java path
#[command]
pub async fn get_bundled_java_path() -> Result<String, String> {
    get_bundled_java_path_for_version(17).await
}

/// Get bundled Java path for specific version
#[command] 
pub async fn get_bundled_java_path_for_version(major_version: u32) -> Result<String, String> {
    crate::minecraft::versions::get_java_for_version(major_version).await
}

/// Download and install Java 17 (default)
#[command]
pub async fn download_and_install_java(app_handle: tauri::AppHandle) -> Result<String, String> {
    download_and_install_java_version(17, app_handle).await
}

/// Download and install Java 8
#[command]
pub async fn download_and_install_java8(app_handle: tauri::AppHandle) -> Result<String, String> {
    download_and_install_java_version(8, app_handle).await
}

/// Download and install Java 17
#[command]
pub async fn download_and_install_java17(app_handle: tauri::AppHandle) -> Result<String, String> {
    download_and_install_java_version(17, app_handle).await
}

/// Download and install both Java 8 and Java 17
#[command]
pub async fn download_and_install_both_java(app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut results = Vec::new();
    
    println!("📦 Installing Java 8 and Java 17...");
    
    // Install Java 8 first
    match download_and_install_java_version(8, app_handle.clone()).await {
        Ok(path) => {
            println!("✓ Java 8 installed successfully at: {}", path);
            results.push(format!("Java 8: {}", path));
        }
        Err(e) => {
            println!("❌ Java 8 installation failed: {}", e);
            results.push(format!("Java 8: Failed - {}", e));
        }
    }
    
    // Install Java 17
    match download_and_install_java_version(17, app_handle).await {
        Ok(path) => {
            println!("✓ Java 17 installed successfully at: {}", path);
            results.push(format!("Java 17: {}", path));
            Ok(results.join("; "))
        }
        Err(e) => {
            println!("❌ Java 17 installation failed: {}", e);
            if results.is_empty() {
                Err(format!("Both Java installations failed. Java 17 error: {}", e))
            } else {
                Ok(format!("{}; Java 17: Failed - {}", results[0], e))
            }
        }
    }
}

/// Download and install specific Java version
#[command]
pub async fn download_and_install_java_version(major_version: u32, app_handle: tauri::AppHandle) -> Result<String, String> {
    use std::fs;
    
    println!("🚀 Starting Java {} installation...", major_version);
    
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java").join(format!("java{}", major_version));
    
    // Check if already installed
    #[cfg(target_os = "windows")]
    let java_exe = java_dir.join("bin").join("java.exe");
    #[cfg(not(target_os = "windows"))]
    let java_exe = java_dir.join("bin").join("java");
    
    if java_exe.exists() {
        println!("✓ Java {} already installed at: {}", major_version, java_exe.display());
        return Ok(java_exe.to_string_lossy().to_string());
    }
    
    // Create directories
    fs::create_dir_all(&java_dir)
        .map_err(|e| format!("Failed to create Java directory: {}", e))?;
    
    // Get download URL
    let download_url = get_java_download_url(major_version)?;
    println!("📥 Downloading from: {}", download_url);
    
    // Download Java
    let temp_file = java_dir.join(format!("java{}_temp.zip", major_version));
    download_file_with_progress(&download_url, &temp_file, &app_handle).await
        .map_err(|e| format!("Failed to download Java {}: {}", major_version, e))?;
    
    // Emit extraction progress
    let _ = app_handle.emit("java_install_progress", serde_json::json!({
        "stage": "Extracting Java...",
        "progress": 85
    }));
    
    println!("📦 Extracting Java {}...", major_version);
    
    // Extract Java
    extract_java_archive(&temp_file, &java_dir, &app_handle).await
        .map_err(|e| format!("Failed to extract Java {}: {}", major_version, e))?;
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file);
    
    // Find the actual Java executable in extracted directories
    let actual_java_exe = find_java_executable(&java_dir)?;
    
    // Emit completion
    let _ = app_handle.emit("java_install_progress", serde_json::json!({
        "stage": "Installation complete!",
        "progress": 100
    }));
    
    println!("✅ Java {} installation completed successfully at: {}", major_version, actual_java_exe.display());
    Ok(actual_java_exe.to_string_lossy().to_string())
}

/// Get Java installations
#[command]
pub async fn get_java_installations() -> Result<Vec<String>, String> {
    let mut java_paths = Vec::new();
    
    // Try to find Java 8 and 17
    for version in [8, 17] {
        if let Ok(java_path) = crate::minecraft::versions::get_java_for_version(version).await {
            java_paths.push(format!("Java {}: {}", version, java_path));
        } else {
            java_paths.push(format!("Java {}: Not installed", version));
        }
    }
    
    Ok(java_paths)
}

/// Get required Java version for a Minecraft version
#[command]
pub async fn get_required_java_version(minecraft_version: String) -> Result<u32, String> {
    Ok(crate::minecraft::versions::get_required_java_version(&minecraft_version))
}

/// Get Java path for a specific Minecraft version
#[command]
pub async fn get_java_for_minecraft_version(minecraft_version: String) -> Result<String, String> {
    let required_java_version = crate::minecraft::versions::get_required_java_version(&minecraft_version);
    println!("Minecraft {} requires Java {}", minecraft_version, required_java_version);
    crate::minecraft::versions::get_java_for_version(required_java_version).await
}

/// Check if Java version is installed
#[command]
pub async fn is_java_version_installed(major_version: u32) -> Result<bool, String> {
    match crate::minecraft::versions::get_java_for_version(major_version).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Validate Java installation
#[command]
pub async fn validate_java_installation(java_path: String) -> Result<String, String> {
    let output = Command::new(&java_path)
        .arg("-version")
        .output()
        .map_err(|e| format!("Failed to execute Java: {}", e))?;
    
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(stderr.to_string())
    } else {
        Err("Java validation failed".to_string())
    }
}

/// Get system memory
#[command]
pub async fn get_system_memory() -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use winapi::um::sysinfoapi::{GetPhysicallyInstalledSystemMemory, MEMORYSTATUSEX, GlobalMemoryStatusEx};
        
        unsafe {
            let mut memory_kb = 0u64;
            if GetPhysicallyInstalledSystemMemory(&mut memory_kb) != 0 {
                Ok(memory_kb / 1024) // Convert to MB
            } else {
                // Fallback to GlobalMemoryStatusEx
                let mut mem_status = MEMORYSTATUSEX {
                    dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
                    ..mem::zeroed()
                };
                
                if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                    Ok((mem_status.ullTotalPhys / (1024 * 1024)) as u64)
                } else {
                    Err("Failed to get system memory".to_string())
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows systems, try reading /proc/meminfo
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Ok(kb / 1024); // Convert to MB
                        }
                    }
                }
            }
        }
        
        // Fallback to a reasonable default
        Ok(8192) // 8GB default
    }
}

/// Download Minecraft assets
#[command]
pub async fn download_minecraft_assets(version: String, game_dir: String) -> Result<(), String> {
    let game_path = PathBuf::from(game_dir);
    let assets_dir = game_path.join("assets");
    
    // Create assets directory structure
    fs::create_dir_all(&assets_dir).await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    
    // Load version manifest to get asset index
    if let Ok(Some(version_json)) = load_version_manifest(&game_path, &version).await {
        if let Some(asset_index) = version_json.get("assetIndex") {
            let index_id = asset_index.get("id").and_then(|v| v.as_str()).unwrap_or(&version);
            let index_url = asset_index.get("url").and_then(|v| v.as_str());
            
            if let Some(url) = index_url {
                println!("📥 Downloading asset index for {}", version);
                
                // Download asset index
                let indexes_dir = assets_dir.join("indexes");
                fs::create_dir_all(&indexes_dir).await
                    .map_err(|e| format!("Failed to create indexes directory: {}", e))?;
                
                let index_file = indexes_dir.join(format!("{}.json", index_id));
                let response = reqwest::get(url).await
                    .map_err(|e| format!("Failed to download asset index: {}", e))?;
                
                let index_content = response.text().await
                    .map_err(|e| format!("Failed to read asset index: {}", e))?;
                
                fs::write(&index_file, &index_content).await
                    .map_err(|e| format!("Failed to write asset index: {}", e))?;
                
                // Parse and download assets
                if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        let objects_dir = assets_dir.join("objects");
                        fs::create_dir_all(&objects_dir).await
                            .map_err(|e| format!("Failed to create objects directory: {}", e))?;
                        
                        let mut downloaded = 0;
                        let total = objects.len();
                        
                        for (_name, asset_info) in objects.iter() {
                            if let Some(hash) = asset_info.get("hash").and_then(|v| v.as_str()) {
                                let hash_prefix = &hash[0..2];
                                let object_dir = objects_dir.join(hash_prefix);
                                let object_file = object_dir.join(hash);
                                
                                if !object_file.exists() {
                                    fs::create_dir_all(&object_dir).await
                                        .map_err(|e| format!("Failed to create object directory: {}", e))?;
                                    
                                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
                                    
                                    if let Ok(response) = reqwest::get(&asset_url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let _ = fs::write(&object_file, &bytes).await;
                                        }
                                    }
                                }
                            }
                            downloaded += 1;
                            
                            if downloaded % 50 == 0 {
                                println!("📦 Downloaded {}/{} assets", downloaded, total);
                            }
                        }
                        
                        println!("✓ Downloaded {} assets for {}", total, version);
                    }
                }
            }
        }
    }
    
    Ok(())
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
    
    let versions_dir = instance_dir.join("versions").join(&version_id);
    fs::create_dir_all(&versions_dir).await
        .map_err(|e| format!("Failed to create version directory: {}", e))?;
    
    // Get version manifest from Mojang
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "manifest", 
        "progress": 10,
        "currentFile": "version_manifest_v2.json",
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest_response = reqwest::get(manifest_url).await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    
    let manifest: VersionManifest = manifest_response.json().await
        .map_err(|e| format!("Failed to parse version manifest: {}", e))?;
    
    // Find the specific version
    let version_info = manifest.versions.iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {} not found", version_id))?;
    
    println!("📥 Downloading version JSON for {}", version_id);
    let _ = app_handle.emit("install_progress", serde_json::json!({
        "instanceId": instance_id,
        "stage": "version_json",
        "progress": 20,
        "currentFile": format!("{}.json", version_id),
        "bytesDownloaded": 0,
        "totalBytes": 0
    }));
    
    // Download version JSON
    let version_response = reqwest::get(&version_info.url).await
        .map_err(|e| format!("Failed to download version JSON: {}", e))?;
    
    let version_json_content = version_response.text().await
        .map_err(|e| format!("Failed to read version JSON: {}", e))?;
    
    // Save version JSON
    let version_json_path = versions_dir.join(format!("{}.json", version_id));
    fs::write(&version_json_path, &version_json_content).await
        .map_err(|e| format!("Failed to save version JSON: {}", e))?;
    
    // Parse version JSON for downloads
    let version_json: serde_json::Value = serde_json::from_str(&version_json_content)
        .map_err(|e| format!("Failed to parse version JSON: {}", e))?;
    
    // Download client JAR
    if let Some(downloads) = version_json.get("downloads") {
        if let Some(client) = downloads.get("client") {
            if let Some(client_url) = client.get("url").and_then(|v| v.as_str()) {
                println!("📥 Downloading client JAR for {}", version_id);
                let _ = app_handle.emit("install_progress", serde_json::json!({
                    "instanceId": instance_id,
                    "stage": "client_jar",
                    "progress": 30,
                    "currentFile": format!("{}.jar", version_id),
                    "bytesDownloaded": 0,
                    "totalBytes": 0
                }));
                
                let jar_response = reqwest::get(client_url).await
                    .map_err(|e| format!("Failed to download client JAR: {}", e))?;
                
                let jar_bytes = jar_response.bytes().await
                    .map_err(|e| format!("Failed to read client JAR: {}", e))?;
                
                let jar_path = versions_dir.join(format!("{}.jar", version_id));
                fs::write(&jar_path, &jar_bytes).await
                    .map_err(|e| format!("Failed to save client JAR: {}", e))?;
            }
        }
    }
    
    // Download libraries
    if let Some(libraries) = version_json.get("libraries").and_then(|v| v.as_array()) {
        let libraries_dir = instance_dir.join("libraries");
        fs::create_dir_all(&libraries_dir).await
            .map_err(|e| format!("Failed to create libraries directory: {}", e))?;
        
        println!("📦 Downloading {} libraries for {}", libraries.len(), version_id);
        let _ = app_handle.emit("install_progress", serde_json::json!({
            "instanceId": instance_id,
            "stage": "libraries",
            "progress": 40,
            "currentFile": "libraries",
            "bytesDownloaded": 0,
            "totalBytes": libraries.len() as u64
        }));
        
        for (i, library) in libraries.iter().enumerate() {
            if let Some(name) = library.get("name").and_then(|v| v.as_str()) {
                // Check if library should be included based on rules
                if !should_include_library(library) {
                    continue;
                }
                
                // Parse Maven coordinate
                let parts: Vec<&str> = name.split(':').collect();
                if parts.len() >= 3 {
                    let group = parts[0].replace(".", "/");
                    let artifact = parts[1];
                    let version = parts[2];
                    
                    let lib_dir = libraries_dir.join(&group).join(&artifact).join(&version);
                    fs::create_dir_all(&lib_dir).await
                        .map_err(|e| format!("Failed to create library directory: {}", e))?;
                    
                    let jar_name = format!("{}-{}.jar", artifact, version);
                    let jar_path = lib_dir.join(&jar_name);
                    
                    if !jar_path.exists() {
                        // Try to download from downloads section first
                        let mut downloaded = false;
                        
                        if let Some(downloads) = library.get("downloads") {
                            if let Some(artifact_info) = downloads.get("artifact") {
                                if let Some(url) = artifact_info.get("url").and_then(|v| v.as_str()) {
                                    if let Ok(response) = reqwest::get(url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let _ = fs::write(&jar_path, &bytes).await;
                                            downloaded = true;
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Fallback to Maven Central
                        if !downloaded {
                            let maven_url = format!(
                                "https://repo1.maven.org/maven2/{}/{}/{}/{}",
                                group, artifact, version, jar_name
                            );
                            
                            if let Ok(response) = reqwest::get(&maven_url).await {
                                if response.status().is_success() {
                                    if let Ok(bytes) = response.bytes().await {
                                        let _ = fs::write(&jar_path, &bytes).await;
                                    }
                                }
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
    
    download_minecraft_assets_with_progress(version_id.clone(), instance_dir.to_string_lossy().to_string(), &instance_id, &app_handle).await?;
    
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
        }
    }
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
        
        // Fallback with just the manual token
        return Ok(AuthInfo {
            username: "Player".to_string(),
            uuid: "12345678-90ab-cdef-1234-567890abcdef".to_string(),
            access_token: token,
            user_type: "msa".to_string(),
        });
    }
    
    // Last resort: Use offline credentials (this should show a warning)
    println!("⚠️  No valid authentication found - launching in offline mode");
    println!("⚠️  Please sign in with a Microsoft account for online play");
    Ok(AuthInfo::default())
}

/// Get authentication info for debugging (public version of get_auth_info)
pub async fn get_auth_info_debug() -> Result<AuthInfo, String> {
    get_auth_info().await
}

/// Download Minecraft assets with progress tracking
pub async fn download_minecraft_assets_with_progress(version: String, game_dir: String, instance_id: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    println!("🚀 Starting asset download for {} in {}", version, game_dir);
    
    let game_path = PathBuf::from(&game_dir);
    let assets_dir = game_path.join("assets");
    
    println!("📂 Assets directory: {}", assets_dir.display());
    
    // Create assets directory structure
    fs::create_dir_all(&assets_dir).await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
        
    println!("✅ Assets directory created successfully");
    
    // Load version manifest to get asset index
    println!("🔍 Loading version manifest for {}", version);
    match load_version_manifest(&game_path, &version).await {
        Ok(Some(version_json)) => {
            println!("✅ Version manifest loaded successfully");
            if let Some(asset_index) = version_json.get("assetIndex") {
                println!("🎯 Found assetIndex in manifest");
                let index_id = asset_index.get("id").and_then(|v| v.as_str()).unwrap_or(&version);
                let index_url = asset_index.get("url").and_then(|v| v.as_str());
                
                println!("📋 Asset index ID: {}", index_id);
                println!("🌐 Asset index URL: {:?}", index_url);
                
                if let Some(url) = index_url {
                    println!("📥 Downloading asset index for {}", version);
                
                // Download asset index
                let indexes_dir = assets_dir.join("indexes");
                fs::create_dir_all(&indexes_dir).await
                    .map_err(|e| format!("Failed to create indexes directory: {}", e))?;
                
                let index_file = indexes_dir.join(format!("{}.json", index_id));
                let response = reqwest::get(url).await
                    .map_err(|e| format!("Failed to download asset index: {}", e))?;
                
                let index_content = response.text().await
                    .map_err(|e| format!("Failed to read asset index: {}", e))?;
                
                fs::write(&index_file, &index_content).await
                    .map_err(|e| format!("Failed to write asset index: {}", e))?;
                
                // Parse and download assets with live progress
                if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        let objects_dir = assets_dir.join("objects");
                        fs::create_dir_all(&objects_dir).await
                            .map_err(|e| format!("Failed to create objects directory: {}", e))?;
                        
                        let mut downloaded = 0;
                        let total = objects.len();
                        
                        for (_name, asset_info) in objects.iter() {
                            if let Some(hash) = asset_info.get("hash").and_then(|v| v.as_str()) {
                                let hash_prefix = &hash[0..2];
                                let object_dir = objects_dir.join(hash_prefix);
                                let object_file = object_dir.join(hash);
                                
                                if !object_file.exists() {
                                    fs::create_dir_all(&object_dir).await
                                        .map_err(|e| format!("Failed to create object directory: {}", e))?;
                                    
                                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
                                    
                                    if let Ok(response) = reqwest::get(&asset_url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let _ = fs::write(&object_file, &bytes).await;
                                        }
                                    }
                                }
                            }
                            downloaded += 1;
                            
                            // Send live asset progress every 50 assets to avoid flooding
                            if downloaded % 50 == 0 || downloaded == total {
                                let progress = 75 + ((downloaded as f64 / total as f64) * 20.0) as u32;
                                let _ = app_handle.emit("install_progress", serde_json::json!({
                                    "instanceId": instance_id,
                                    "stage": "assets", 
                                    "progress": progress,
                                    "currentFile": format!("asset_{}", downloaded),
                                    "bytesDownloaded": downloaded as u64,
                                    "totalBytes": total as u64
                                }));
                            }
                        }
                        
                        println!("✓ Downloaded {} assets for {}", total, version);
                    } else {
                        println!("❌ No objects found in asset index");
                    }
                } else {
                    println!("❌ Failed to parse asset index JSON");
                }
            } else {
                println!("❌ No asset index URL found");
            }
        } else {
            println!("❌ No assetIndex found in version manifest");
        }
    },
    Ok(None) => {
        println!("❌ Version manifest not found for {}", version);
    },
    Err(e) => {
        println!("❌ Failed to load version manifest: {:?}", e);
    }
}
    
    println!("🏁 Asset download completed for {}", version);
    Ok(())
}

// Helper functions for Java installation

/// Get download URL for Java version from Eclipse Temurin
fn get_java_download_url(major_version: u32) -> Result<String, String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err("Unsupported architecture".to_string());
    };
    
    let _file_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    
    // Use Eclipse Temurin API to get latest version
    Ok(format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
        major_version, os, arch
    ))
}

/// Download file with progress tracking
async fn download_file_with_progress(
    url: &str,
    dest: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    println!("📥 Downloading from: {}", url);
    
    let response = reqwest::get(url).await
        .map_err(|e| format!("Failed to start download: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    
    let mut file = std::fs::File::create(dest)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    let mut stream = response.bytes_stream();
    use futures::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 80.0; // Reserve 20% for extraction
            let _ = app_handle.emit("java_install_progress", serde_json::json!({
                "stage": "Downloading Java...",
                "progress": progress as u32
            }));
        }
    }
    
    println!("✓ Download completed: {}", dest.display());
    Ok(())
}

/// Extract Java archive (ZIP on Windows, tar.gz on Unix)
async fn extract_java_archive(archive_path: &PathBuf, extract_dir: &PathBuf, app_handle: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use zip::ZipArchive;
        use std::fs::File;
        
        let file = File::open(archive_path)
            .map_err(|e| format!("Failed to open archive: {}", e))?;
        
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
        
        // Extract all files in a blocking manner to avoid async/lifetime issues
        let total_files = archive.len();
        let extract_dir_clone = extract_dir.clone();
        let app_handle_clone = app_handle.clone();
        
        // Use spawn_blocking to handle the ZIP extraction synchronously
        tokio::task::spawn_blocking(move || {
            for i in 0..total_files {
                let mut file = archive.by_index(i)
                    .map_err(|e| format!("Failed to extract file {}: {}", i, e))?;
                
                let outpath = match file.enclosed_name() {
                    Some(path) => extract_dir_clone.join(path),
                    None => continue,
                };
                
                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                    }
                    
                    let mut output_file = std::fs::File::create(&outpath)
                        .map_err(|e| format!("Failed to create output file: {}", e))?;
                    std::io::copy(&mut file, &mut output_file)
                        .map_err(|e| format!("Failed to copy file: {}", e))?;
                }
                
                // Emit extraction progress (80% to 95%)
                if i % 50 == 0 {
                    let extract_progress = 80 + ((i as f64 / total_files as f64) * 15.0) as u32;
                    let _ = app_handle_clone.emit("java_install_progress", serde_json::json!({
                        "stage": format!("Extracting... ({}/{})", i + 1, total_files),
                        "progress": extract_progress
                    }));
                }
            }
            Ok::<(), String>(())
        }).await.map_err(|e| format!("Extraction task failed: {}", e))??;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Stdio;
        
        let output = tokio::process::Command::new("tar")
            .args(&["-xzf", &archive_path.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute tar: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Tar extraction failed: {}", stderr));
        }
    }
    
    // Remove the downloaded archive
    let _ = fs::remove_file(archive_path).await;
    println!("✓ Java extracted successfully");
    
    Ok(())
}

/// Find Java executable in extracted directory (handles nested JDK directories)
fn find_java_executable(java_dir: &PathBuf) -> Result<PathBuf, String> {
    use std::fs;
    
    #[cfg(target_os = "windows")]
    let java_exe_name = "java.exe";
    #[cfg(not(target_os = "windows"))]
    let java_exe_name = "java";
    
    // First try direct path (java_dir/bin/java.exe)
    let direct_path = java_dir.join("bin").join(java_exe_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }
    
    // Search for nested JDK directories (like jdk-17.0.16+8)
    let entries = fs::read_dir(java_dir)
        .map_err(|e| format!("Failed to read Java directory: {}", e))?;
        
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let potential_java = path.join("bin").join(java_exe_name);
            if potential_java.exists() {
                return Ok(potential_java);
            }
        }
    }
    
    Err(format!("Java executable not found in {}", java_dir.display()))
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

