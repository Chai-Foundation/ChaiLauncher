use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::command;
use tokio::fs;
use tokio::process::Command;
use crate::launchers::{LauncherManager, ExternalInstance};
use crate::storage::{StorageManager, InstanceMetadata};
use crate::installer::MinecraftInstaller;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub modpack: Option<String>,
    pub modpack_version: Option<String>,
    pub game_dir: PathBuf,
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub last_played: Option<String>,
    pub total_play_time: u64,
    pub icon: Option<String>,
    pub is_modded: bool,
    pub mods_count: u32,
    pub is_external: Option<bool>,
    pub external_launcher: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: HashMap<String, String>,
    pub versions: Vec<MinecraftVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherProfile {
    pub name: String,
    pub game_dir: String,
    pub java_executable: Option<String>,
    pub java_args: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub jvm_args: Option<String>,
    pub last_version_id: String,
    pub icon: Option<String>,
    pub created: String,
    pub last_used: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherProfiles {
    pub profiles: HashMap<String, LauncherProfile>,
    pub selected_profile: Option<String>,
    pub client_token: String,
    pub launcher_version: String,
}

#[command]
pub async fn get_minecraft_versions() -> Result<VersionManifest, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    
    let manifest: VersionManifest = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse version manifest: {}", e))?;
    
    Ok(manifest)
}

#[command]
pub async fn create_instance(
    name: String,
    version: String,
    game_dir: String,
    modpack: Option<String>,
    modpack_version: Option<String>,
) -> Result<MinecraftInstance, String> {
    let instance_dir = PathBuf::from(&game_dir).join(&name);
    
    // Create instance directory
    fs::create_dir_all(&instance_dir)
        .await
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    let instance = MinecraftInstance {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        version,
        modpack,
        modpack_version,
        game_dir: instance_dir,
        java_path: None,
        jvm_args: None,
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: false,
        mods_count: 0,
        is_external: Some(false),
        external_launcher: None,
    };
    
    Ok(instance)
}

#[command]
pub async fn launch_minecraft(
    instance: MinecraftInstance,
    java_path: Option<String>,
    max_memory: Option<u32>,
    min_memory: Option<u32>,
    additional_args: Option<Vec<String>>,
) -> Result<(), String> {
    let java_executable = java_path.unwrap_or_else(|| "java".to_string());
    
    let mut args = Vec::new();
    
    // Memory arguments
    if let Some(max_mem) = max_memory {
        args.push(format!("-Xmx{}M", max_mem));
    }
    if let Some(min_mem) = min_memory {
        args.push(format!("-Xms{}M", min_mem));
    }
    
    // JVM arguments
    if let Some(jvm_args) = &instance.jvm_args {
        args.extend(jvm_args.clone());
    }
    
    // Additional arguments
    if let Some(additional) = additional_args {
        args.extend(additional);
    }
    
    // Basic Minecraft launch arguments (simplified)
    args.extend([
        "-cp".to_string(),
        "minecraft.jar".to_string(), // This would be the full classpath in reality
        "net.minecraft.client.main.Main".to_string(),
        "--version".to_string(),
        instance.version.clone(),
        "--gameDir".to_string(),
        instance.game_dir.to_string_lossy().to_string(),
    ]);
    
    println!("Launching Minecraft with command: {} {}", java_executable, args.join(" "));
    
    let _child = Command::new(java_executable)
        .args(&args)
        .current_dir(&instance.game_dir)
        .spawn()
        .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_java_installations() -> Result<Vec<String>, String> {
    let mut java_paths = Vec::new();
    
    // Common Java installation paths on different platforms
    #[cfg(target_os = "windows")]
    {
        let common_paths = [
            "C:\\Program Files\\Java",
            "C:\\Program Files (x86)\\Java",
            "C:\\Program Files\\Eclipse Adoptium",
            "C:\\Program Files\\Microsoft\\jdk",
        ];
        
        for path in &common_paths {
            if let Ok(entries) = fs::read_dir(path).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if entry.file_type().await.map_or(false, |ft| ft.is_dir()) {
                        let java_exe = entry.path().join("bin\\java.exe");
                        if java_exe.exists() {
                            java_paths.push(java_exe.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let common_paths = [
            "/Library/Java/JavaVirtualMachines",
            "/System/Library/Java/JavaVirtualMachines",
        ];
        
        for path in &common_paths {
            if let Ok(entries) = fs::read_dir(path).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if entry.file_type().await.map_or(false, |ft| ft.is_dir()) {
                        let java_exe = entry.path().join("Contents/Home/bin/java");
                        if java_exe.exists() {
                            java_paths.push(java_exe.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let common_paths = [
            "/usr/lib/jvm",
            "/usr/java",
            "/opt/java",
        ];
        
        for path in &common_paths {
            if let Ok(entries) = fs::read_dir(path).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if entry.file_type().await.map_or(false, |ft| ft.is_dir()) {
                        let java_exe = entry.path().join("bin/java");
                        if java_exe.exists() {
                            java_paths.push(java_exe.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(java_paths)
}

#[command]
pub async fn validate_java_installation(java_path: String) -> Result<String, String> {
    let output = Command::new(&java_path)
        .arg("-version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute Java: {}", e))?;
    
    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stderr);
        Ok(version_output.to_string())
    } else {
        Err("Java executable is not valid".to_string())
    }
}

#[command]
pub async fn get_system_memory() -> Result<u64, String> {
    // Get system memory in MB
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use winapi::um::sysinfoapi::{MEMORYSTATUSEX, GlobalMemoryStatusEx};
        
        unsafe {
            let mut mem_info: MEMORYSTATUSEX = mem::zeroed();
            mem_info.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
            
            if GlobalMemoryStatusEx(&mut mem_info) != 0 {
                Ok(mem_info.ullTotalPhys / 1024 / 1024) // Convert to MB
            } else {
                Err("Failed to get system memory information".to_string())
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows systems, use a simple fallback
        Ok(8192) // Default to 8GB
    }
}

#[command]
pub async fn download_minecraft_assets(version: String, game_dir: String) -> Result<(), String> {
    // This is a simplified version - in reality, you'd need to:
    // 1. Download the version JSON from the version manifest
    // 2. Parse the JSON to get asset information
    // 3. Download libraries and assets
    // 4. Set up the proper directory structure
    
    let assets_dir = PathBuf::from(&game_dir).join("assets");
    fs::create_dir_all(&assets_dir)
        .await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    
    // Create basic directory structure
    let dirs = ["libraries", "versions", "assets/indexes", "assets/objects"];
    for dir in &dirs {
        let dir_path = PathBuf::from(&game_dir).join(dir);
        fs::create_dir_all(&dir_path)
            .await
            .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
    }
    
    println!("Created basic Minecraft directory structure for version {}", version);
    Ok(())
}

// Legacy GDLauncherInstance struct - now using ExternalInstance from launchers module

#[command]
pub async fn detect_all_external_instances() -> Result<Vec<ExternalInstance>, String> {
    let launcher_manager = LauncherManager::new();
    launcher_manager.detect_all_instances().await
}

#[command]
pub async fn detect_gdlauncher_instances() -> Result<Vec<ExternalInstance>, String> {
    let launcher_manager = LauncherManager::new();
    let all_instances = launcher_manager.detect_all_instances().await?;
    
    // Filter only GDLauncher instances for backward compatibility
    Ok(all_instances.into_iter()
        .filter(|instance| instance.launcher_type == "gdlauncher")
        .collect())
}

#[command]
pub async fn launch_instance(
    instance_id: String,
    instance_path: String,
    version: String,
    java_path: String,
    memory: u32,
    jvm_args: Vec<String>,
) -> Result<(), String> {
    // Check if this is an external launcher instance
    if instance_id.contains("-") {
        let launcher_type = instance_id.split("-").next().unwrap_or("");
        match launcher_type {
            "gdl" | "multimc" | "prism" | "modrinth" => {
                return launch_external_instance(instance_id, instance_path).await;
            }
            _ => {}
        }
    }
    
    // For our own instances, use the existing launch_minecraft function
    let instance = MinecraftInstance {
        id: instance_id,
        name: "Instance".to_string(),
        version,
        modpack: None,
        modpack_version: None,
        game_dir: PathBuf::from(&instance_path),
        java_path: Some(java_path),
        jvm_args: Some(jvm_args),
        last_played: None,
        total_play_time: 0,
        icon: None,
        is_modded: false,
        mods_count: 0,
        is_external: None,
        external_launcher: None,
    };
    
    launch_minecraft(instance, None, Some(memory), None, None).await
}

#[command]
pub async fn launch_external_instance(instance_id: String, instance_path: String) -> Result<(), String> {
    let launcher_manager = LauncherManager::new();
    let launcher_type = instance_id.split("-").next().unwrap_or("");
    
    if launcher_manager.get_launcher_for_instance(launcher_type).await.is_some() {
        // Launch using the appropriate launcher
        match launcher_type {
            "gdl" => launch_gdlauncher_instance(instance_id, instance_path).await,
            "multimc" => launch_multimc_instance(instance_id, instance_path).await,
            "prism" => launch_prism_instance(instance_id, instance_path).await,
            "modrinth" => launch_modrinth_instance(instance_id, instance_path).await,
            _ => Err(format!("Unknown launcher type: {}", launcher_type))
        }
    } else {
        Err(format!("Launcher not found: {}", launcher_type))
    }
}

async fn get_gdlauncher_config_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Could not get APPDATA environment variable")?;
        Ok(PathBuf::from(appdata).join("gdlauncher_next"))
    }
    
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "Could not get HOME environment variable")?;
        Ok(PathBuf::from(home).join("Library/Application Support/gdlauncher_next"))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "Could not get HOME environment variable")?;
        Ok(PathBuf::from(home).join(".local/share/gdlauncher_next"))
    }
}


async fn launch_gdlauncher_instance(instance_id: String, _instance_path: String) -> Result<(), String> {
    use crate::launchers::gdlauncher::GDLauncherDetector;
    use crate::launchers::LauncherLauncher;
    
    let detector = GDLauncherDetector;
    detector.launch_instance(&instance_id, &_instance_path).await
}

async fn launch_multimc_instance(instance_id: String, _instance_path: String) -> Result<(), String> {
    use crate::launchers::multimc::MultiMCDetector;
    use crate::launchers::LauncherLauncher;
    
    let detector = MultiMCDetector;
    detector.launch_instance(&instance_id, &_instance_path).await
}

async fn launch_prism_instance(instance_id: String, _instance_path: String) -> Result<(), String> {
    use crate::launchers::prism::PrismDetector;
    use crate::launchers::LauncherLauncher;
    
    let detector = PrismDetector;
    detector.launch_instance(&instance_id, &_instance_path).await
}

async fn launch_modrinth_instance(instance_id: String, _instance_path: String) -> Result<(), String> {
    use crate::launchers::modrinth::ModrinthDetector;
    use crate::launchers::LauncherLauncher;
    
    let detector = ModrinthDetector;
    detector.launch_instance(&instance_id, &_instance_path).await
}

async fn find_gdlauncher_executable() -> Result<String, String> {
    use crate::launchers::gdlauncher::GDLauncherDetector;
    use crate::launchers::LauncherDetector;
    
    let detector = GDLauncherDetector;
    match detector.get_executable_path().await? {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("GDLauncher executable not found".to_string())
    }
}

// New storage-based commands

#[command]
pub async fn load_instances() -> Result<Vec<InstanceMetadata>, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    Ok(storage.get_all_instances().into_iter().cloned().collect())
}

#[command]
pub async fn save_instance(instance: InstanceMetadata) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.add_instance(instance).await
        .map_err(|e| format!("Failed to save instance: {}", e))
}

#[command]
pub async fn delete_instance(instance_id: String) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.remove_instance(&instance_id).await
        .map_err(|e| format!("Failed to delete instance: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn update_instance(instance: InstanceMetadata) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.update_instance(instance).await
        .map_err(|e| format!("Failed to update instance: {}", e))
}

#[command]
pub async fn get_launcher_settings() -> Result<crate::storage::LauncherSettings, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    Ok(storage.get_settings().clone())
}

#[command]
pub async fn update_launcher_settings(settings: crate::storage::LauncherSettings) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.update_settings(settings).await
        .map_err(|e| format!("Failed to update settings: {}", e))
}

#[command]
pub async fn install_minecraft_version(
    version_id: String,
    instance_name: String,
    game_dir: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let instance_id = uuid::Uuid::new_v4().to_string();
    let instance_dir = PathBuf::from(&game_dir).join(&instance_name);
    
    // Create instance directory
    fs::create_dir_all(&instance_dir).await
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;

    // Initialize installer
    let installer = MinecraftInstaller::new(instance_dir.clone());
    
    // Install with progress tracking
    let app_handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();
    
    installer.install_version(&version_id, move |progress| {
        let _ = app_handle_clone.emit("install_progress", InstallProgressEvent {
            instance_id: instance_id_clone.clone(),
            stage: progress.stage,
            progress: progress.progress,
            current_file: progress.current_file,
            bytes_downloaded: progress.bytes_downloaded,
            total_bytes: progress.total_bytes,
        });
    }).await.map_err(|e| format!("Installation failed: {}", e))?;

    // Create instance metadata
    let instance = InstanceMetadata {
        id: instance_id.clone(),
        name: instance_name,
        version: version_id,
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
        tags: Vec::new(),
    };

    // Save instance
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.add_instance(instance).await
        .map_err(|e| format!("Failed to save instance: {}", e))?;

    // Emit completion event
    let _ = app_handle.emit("install_complete", InstallCompleteEvent {
        instance_id: instance_id.clone(),
        success: true,
        error: None,
    });

    Ok(instance_id)
}

#[command]
pub async fn backup_instance(instance_id: String, backup_path: String) -> Result<(), String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.backup_instance(&instance_id, &PathBuf::from(backup_path)).await
        .map_err(|e| format!("Backup failed: {}", e))
}

#[command]
pub async fn restore_instance(instance_id: String, backup_path: String) -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.restore_instance(&PathBuf::from(backup_path), &instance_id).await
        .map_err(|e| format!("Restore failed: {}", e))
}

#[command]
pub async fn refresh_instance_sizes() -> Result<(), String> {
    let mut storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    storage.refresh_instance_sizes().await
        .map_err(|e| format!("Failed to refresh sizes: {}", e))
}

// Events for frontend
#[derive(Debug, Serialize, Clone)]
pub struct InstallProgressEvent {
    pub instance_id: String,
    pub stage: String,
    pub progress: f64,
    pub current_file: Option<String>,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct InstallCompleteEvent {
    pub instance_id: String,
    pub success: bool,
    pub error: Option<String>,
}