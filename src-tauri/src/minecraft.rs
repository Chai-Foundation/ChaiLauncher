use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{command, Manager};
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
pub async fn get_bundled_java_path() -> Result<String, String> {
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java");
    
    #[cfg(target_os = "windows")]
    let java_exe = java_dir.join("bin").join("java.exe");
    
    #[cfg(not(target_os = "windows"))]
    let java_exe = java_dir.join("bin").join("java");
    
    if java_exe.exists() {
        Ok(java_exe.to_string_lossy().to_string())
    } else {
        Err("Bundled Java not found".to_string())
    }
}

#[command]
pub async fn download_and_install_java(app_handle: tauri::AppHandle) -> Result<String, String> {
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java");
    
    // Create java directory
    fs::create_dir_all(&java_dir).await
        .map_err(|e| format!("Failed to create Java directory: {}", e))?;
    
    // For now, we'll use a placeholder - in production you'd download from Adoptium/Eclipse Temurin
    // or bundle Java with the installer
    #[cfg(target_os = "windows")]
    {
        // Windows Java 17 download URL (example)
        let java_url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.8.1%2B1/OpenJDK17U-jdk_x64_windows_hotspot_17.0.8.1_1.zip";
        download_and_extract_java(java_url, &java_dir, app_handle.clone()).await?;
    }
    
    #[cfg(target_os = "macos")]
    {
        let java_url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.8.1%2B1/OpenJDK17U-jdk_x64_mac_hotspot_17.0.8.1_1.tar.gz";
        download_and_extract_java(java_url, &java_dir, app_handle.clone()).await?;
    }
    
    #[cfg(target_os = "linux")]
    {
        let java_url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.8.1%2B1/OpenJDK17U-jdk_x64_linux_hotspot_17.0.8.1_1.tar.gz";
        download_and_extract_java(java_url, &java_dir, app_handle.clone()).await?;
    }
    
    get_bundled_java_path().await
}

async fn download_and_extract_java(url: &str, java_dir: &PathBuf, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    
    // Emit start event
    let _ = app_handle.emit("java_install_progress", JavaInstallEvent {
        stage: "Starting download".to_string(),
        progress: 0.0,
    });
    
    // Download Java
    let response = reqwest::get(url).await
        .map_err(|e| format!("Failed to download Java: {}", e))?;
    
    let content_length = response.content_length().unwrap_or(0);
    
    let _ = app_handle.emit("java_install_progress", JavaInstallEvent {
        stage: "Downloading Java Runtime".to_string(),
        progress: 10.0,
    });
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read Java download: {}", e))?;
    
    let _ = app_handle.emit("java_install_progress", JavaInstallEvent {
        stage: "Download complete".to_string(),
        progress: 50.0,
    });
    
    // Save to temp file with proper extension
    #[cfg(target_os = "windows")]
    let temp_file = java_dir.join("java_download.zip");
    
    #[cfg(not(target_os = "windows"))]
    let temp_file = java_dir.join("java_download.tar.gz");
    
    fs::write(&temp_file, &bytes).await
        .map_err(|e| format!("Failed to save Java download: {}", e))?;
    
    let _ = app_handle.emit("java_install_progress", JavaInstallEvent {
        stage: "Extracting Java Runtime".to_string(),
        progress: 70.0,
    });
    
    // Extract based on platform
    #[cfg(target_os = "windows")]
    {
        extract_zip(&temp_file, java_dir).await?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        extract_tar_gz(&temp_file, java_dir).await?;
    }
    
    let _ = app_handle.emit("java_install_progress", JavaInstallEvent {
        stage: "Installation complete".to_string(),
        progress: 100.0,
    });
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file).await;
    
    Ok(())
}

#[cfg(target_os = "windows")]
async fn extract_zip(zip_path: &PathBuf, extract_to: &PathBuf) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;
    
    // Create a temp extraction directory
    let temp_extract = extract_to.join("temp_extract");
    
    // Use PowerShell to extract to temp directory first
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            zip_path.display(),
            temp_extract.display()
        ))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to extract Java: {}", error));
    }
    
    // Find the JDK directory inside temp_extract and move its contents to extract_to
    if let Ok(entries) = std::fs::read_dir(&temp_extract) {
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                let jdk_dir = entry.path();
                // Move contents of JDK directory to the target directory
                move_directory_contents(&jdk_dir, extract_to).await?;
                break;
            }
        }
    }
    
    // Clean up temp directory
    let _ = std::fs::remove_dir_all(&temp_extract);
    
    Ok(())
}

async fn move_directory_contents(source: &PathBuf, target: &PathBuf) -> Result<(), String> {
    use tokio::process::Command;
    
    // Use robocopy to move contents
    let output = Command::new("robocopy")
        .arg(source)
        .arg(target)
        .arg("/E")      // Copy subdirectories including empty ones
        .arg("/MOVE")   // Move files and directories
        .arg("/NFL")    // No file list
        .arg("/NDL")    // No directory list
        .arg("/NJH")    // No job header
        .arg("/NJS")    // No job summary
        .output()
        .await
        .map_err(|e| format!("Failed to execute robocopy: {}", e))?;
    
    // Robocopy returns different exit codes, 0-7 are success
    let exit_code = output.status.code().unwrap_or(8);
    if exit_code > 7 {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to move Java contents: {}", error));
    }
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
async fn extract_tar_gz(tar_path: &PathBuf, extract_to: &PathBuf) -> Result<(), String> {
    use tokio::process::Command;
    
    let output = Command::new("tar")
        .arg("-xzf")
        .arg(tar_path)
        .arg("-C")
        .arg(extract_to)
        .arg("--strip-components=1")
        .output()
        .await
        .map_err(|e| format!("Failed to execute tar: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to extract Java: {}", error));
    }
    
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
    instancePath: String,
    version: String,
    javaPath: String,
    memory: u32,
    jvmArgs: Vec<String>,
) -> Result<(), String> {
    let instance_id = instance_id;
    let instance_path = instancePath;
    let java_path = javaPath;
    let jvm_args = jvmArgs;
    // Validate instance path is not empty
    if instance_path.is_empty() {
        return Err("Instance path cannot be empty".to_string());
    }
    
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
    
    // For our own instances, check if Minecraft is actually installed
    let instance_path_buf = PathBuf::from(&instance_path);
    
    // Check if this instance has been properly set up with Minecraft
    let versions_dir = instance_path_buf.join("versions").join(&version);
    let version_jar = versions_dir.join(format!("{}.jar", version));
    
    if !version_jar.exists() {
        return Err(format!(
            "Minecraft {} is not installed for this instance. Please reinstall the instance to download Minecraft files.", 
            version
        ));
    }
    
    // Check if libraries exist
    let libraries_dir = instance_path_buf.join("libraries");
    if !libraries_dir.exists() {
        return Err(format!(
            "Minecraft libraries are missing for this instance. Please reinstall the instance."
        ));
    }
    
    // For now, return a helpful error message instead of attempting to launch incomplete instances
    Err(format!(
        "Minecraft launching is not fully implemented yet. Instance '{}' is installed but the launcher needs to build the proper classpath and download all required libraries to launch Minecraft {}.",
        instance_id, version
    ))
}

#[command]
pub async fn launch_external_instance(instance_id: String, instance_path: String) -> Result<(), String> {
    // Validate instance path
    if instance_path.is_empty() {
        return Err("Instance path cannot be empty".to_string());
    }
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
pub async fn load_instances() -> Result<Vec<MinecraftInstance>, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let instances: Vec<MinecraftInstance> = storage.get_all_instances()
        .into_iter()
        .cloned()
        .map(|metadata| metadata.into())
        .collect();
    
    println!("Loading {} instances from storage", instances.len());
    for instance in &instances {
        println!("  - Instance: {} ({})", instance.name, instance.id);
    }
    
    Ok(instances)
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
    let instance_id = instance_id;
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
    instance_id: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Validate parameters
    if version_id.is_empty() {
        return Err("version_id cannot be empty".to_string());
    }
    if instance_name.is_empty() {
        return Err("instance_name cannot be empty".to_string());
    }
    if game_dir.is_empty() {
        return Err("game_dir cannot be empty".to_string());
    }
    
    let instance_id = instance_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let instance_dir = PathBuf::from(&game_dir).join(&instance_name);
    
    println!("Creating instance '{}' with ID: {}", instance_name, instance_id);
    
    // Create instance directory
    fs::create_dir_all(&instance_dir).await
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;

    // Initialize installer
    let installer = MinecraftInstaller::new(instance_dir.clone());
    
    // Install with progress tracking
    let app_handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();
    
    println!("Starting installer.install_version for: {}", version_id);
    
    // Small delay to ensure frontend is ready to receive events
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Emit initial progress event
    let event = InstallProgressEvent {
        instance_id: instance_id.clone(),
        stage: "Initializing installation".to_string(),
        progress: 0.0,
        current_file: None,
        bytes_downloaded: 0,
        total_bytes: 0,
    };
    
    println!("Emitting initial install_progress event: {:?}", event);
    
    // Try both global emit and window-specific emit
    let emit_result = app_handle.emit("install_progress", &event);
    if let Err(e) = emit_result {
        println!("Failed to emit initial progress event globally: {}", e);
    }
    
    // Also try emitting to specific window
    if let Some(window) = app_handle.get_webview_window("main") {
        let window_emit_result = window.emit("install_progress", &event);
        if let Err(e) = window_emit_result {
            println!("Failed to emit initial progress event to main window: {}", e);
        } else {
            println!("Successfully emitted to main window");
        }
    } else {
        println!("Main window not found");
    }
    
    installer.install_version(&version_id, move |progress| {
        println!("Progress update: stage={}, progress={:.2}%", progress.stage, progress.progress);
        let event = InstallProgressEvent {
            instance_id: instance_id_clone.clone(),
            stage: progress.stage,
            progress: progress.progress,
            current_file: progress.current_file,
            bytes_downloaded: progress.bytes_downloaded,
            total_bytes: progress.total_bytes,
        };
        
        // Emit to both global and window
        let _ = app_handle_clone.emit("install_progress", &event);
        if let Some(window) = app_handle_clone.get_webview_window("main") {
            let _ = window.emit("install_progress", &event);
        }
    }).await.map_err(|e| {
        let error_msg = format!("Installation failed: {}", e);
        println!("Installation error: {}", error_msg);
        error_msg
    })?;
    
    println!("Installation completed successfully");

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
    let completion_event = InstallCompleteEvent {
        instance_id: instance_id.clone(),
        success: true,
        error: None,
    };
    
    println!("Emitting install_complete event: {:?}", completion_event);
    let _ = app_handle.emit("install_complete", &completion_event);
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("install_complete", &completion_event);
    }

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

#[derive(Debug, Serialize, Clone)]
pub struct JavaInstallEvent {
    pub stage: String,
    pub progress: f64,
}