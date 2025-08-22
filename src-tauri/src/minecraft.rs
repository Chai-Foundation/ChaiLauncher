use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{command, Manager};
use tokio::fs;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::launchers::{LauncherManager, ExternalInstance};
use crate::storage::{StorageManager, InstanceMetadata};
use crate::installer::MinecraftInstaller;
use tauri::Emitter;
use walkdir::WalkDir;

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

#[derive(Debug, Deserialize)]
struct Library {
    downloads: Option<Value>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    id: String,
}

#[derive(Debug, Deserialize)]
struct VersionJson {
    id: String,
    mainClass: String,
    assetIndex: AssetIndex,
    libraries: Vec<Library>,
    arguments: Value,
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

    // Load auth token
    let storage = crate::storage::StorageManager::new()
        .await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;

    let auth_token = if let Some(token) = storage.get_settings().auth_token.clone() {
        Some(token)
    } else {
        crate::auth::get_active_account_token().await.ok().flatten()
    };

    // Load version JSON
    let version_json_path = instance
        .game_dir
        .join(format!("versions/{0}/{0}.json", instance.version));

    let version_json_str = fs::read_to_string(&version_json_path)
        .await
        .map_err(|e| format!("Failed to read version JSON: {}", e))?;

    let version_json: VersionJson = serde_json::from_str(&version_json_str)
        .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

    // JVM arguments
    let mut args = vec![
        format!("-Xmx{}M", max_memory.unwrap_or(4096)),
        format!("-Xms{}M", min_memory.unwrap_or(2048)),
    ];

    // Add natives path
    let natives_path = instance.game_dir.join("natives");
    args.push(format!("-Djava.library.path={}", natives_path.display()));
    args.push(format!("-Djna.library.path={}", natives_path.display()));
    args.push(format!("-Dorg.lwjgl.librarypath={}", natives_path.display()));

    // Build classpath
    let classpath = build_classpath_from_json(&instance, &version_json)?;
    args.push("-cp".to_string());
    args.push(classpath);

    // Add main class from JSON
    args.push(version_json.mainClass.clone());

    // Core arguments
    args.extend(vec![
        "--version".to_string(),
        instance.version.clone(),
        "--gameDir".to_string(),
        instance.game_dir.to_string_lossy().to_string(),
        "--assetsDir".to_string(),
        instance.game_dir.join("assets").to_string_lossy().to_string(),
        "--assetIndex".to_string(),
        version_json.assetIndex.id.clone(),
        "--username".to_string(),
        "Player".to_string(),
        "--uuid".to_string(),
        auth_token.clone().unwrap_or_else(|| "12345678-90ab-cdef-1234-567890abcdef".to_string()),
        "--accessToken".to_string(),
        auth_token.clone().unwrap_or_else(|| "123456".to_string()),
        "--userType".to_string(),
        if auth_token.is_some() { "msa".to_string() } else { "legacy".to_string() },
    ]);

    // Append additional args if provided
    if let Some(extra) = additional_args {
        args.extend(extra);
    }

    println!(
        "Launching Minecraft:\n{} {}",
        java_executable,
        args.join(" ")
    );

    Command::new(&java_executable)
        .args(&args)
        .current_dir(&instance.game_dir)
        .spawn()
        .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;

    Ok(())
}

/// Build classpath from libraries and version jar
fn build_classpath_from_json(instance: &MinecraftInstance, version_json: &VersionJson) -> Result<String, String> {
    let mut entries = Vec::new();

    // Add library jars
    let libraries_path = instance.game_dir.join("libraries");
    for lib in &version_json.libraries {
        let path = lib.name.replace(":", "/");
        let parts: Vec<&str> = lib.name.split(':').collect();
        if parts.len() == 3 {
            let group = parts[0].replace(".", "/");
            let artifact = parts[1];
            let version = parts[2];
            let jar_path = libraries_path
                .join(&group)
                .join(&artifact)
                .join(&version)
                .join(format!("{}-{}.jar", artifact, version));
            if jar_path.exists() {
                entries.push(jar_path.to_string_lossy().to_string());
            } else {
                println!("Warning: Missing library JAR: {}", jar_path.display());
            }
        }
    }

    // Add main version jar
    let main_jar = instance
        .game_dir
        .join(format!("versions/{0}/{0}.jar", instance.version));
    entries.push(main_jar.to_string_lossy().to_string());

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    Ok(entries.join(sep))
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
    
    let _content_length = response.content_length().unwrap_or(0);
    
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
    println!("Checking for Minecraft version directory: {}", versions_dir.display());
    let version_jar = versions_dir.join(format!("{}.jar", version));
    println!("Checking for Minecraft version JAR: {}", version_jar.display());

    
    if !version_jar.exists() {
        return Err(format!(
            "Minecraft {} is not installed for this instance. Please reinstall the instance to download Minecraft files.", 
            version
        ));
    }
    
    // Check if libraries exist
    let libraries_dir = instance_path_buf.join("libraries");
    println!("Checking for Minecraft libraries directory: {}", libraries_dir.display());
    if !libraries_dir.exists() {
        return Err(format!(
            "Minecraft libraries are missing for this instance. Please reinstall the instance."
        ));
    }
    
    // Get auth token from settings or active account
    let storage = crate::storage::StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let auth_token = if let Some(manual_token) = storage.get_settings().auth_token.clone() {
        // Use manual token if set
        Some(manual_token)
    } else {
        // Try to get token from active Microsoft account
        match crate::auth::get_active_account_token().await {
            Ok(token) => token,
            Err(e) => {
                println!("Warning: No auth token available: {}", e);
                None
            }
        }
    };
    
    // Launch Minecraft
    let mut args = Vec::new();

    // Memory arguments
    args.push(format!("-Xmx{}M", memory));
    args.push(format!("-Xms{}M", memory));

    // JVM arguments
    args.extend(jvm_args.clone());
    
    // Add auth token as JVM argument if available
    if let Some(ref token) = auth_token {
        args.push(format!("-Dauth.token={}", token));
        println!("Added auth token to JVM args");
    } else {
        println!("No auth token available for launch");
    }
    
    // Add system properties to fix SLF4J and logging issues
    args.push("-Dlog4j2.formatMsgNoLookups=true".to_string());
    args.push("-Dorg.slf4j.simpleLogger.defaultLogLevel=warn".to_string());
    
    // Use proper path for natives directory with proper escaping
    let natives_path = PathBuf::from(instance_path.clone()).join("natives");
    let natives_path_str = natives_path.to_string_lossy();
    
    // Additional Minecraft-specific system properties
    args.push(format!("-Djava.library.path={}", natives_path_str));
    args.push("-Dminecraft.launcher.brand=ChaiLauncher".to_string());
    args.push("-Dminecraft.launcher.version=1.0.0".to_string());
    
    // Graphics and performance improvements
    args.push("-Dfile.encoding=UTF-8".to_string());
    args.push("-Dsun.stderr.encoding=UTF-8".to_string());
    args.push("-Dsun.stdout.encoding=UTF-8".to_string());
    
    // JNA (Java Native Access) configuration
    args.push(format!("-Djna.library.path={}", natives_path.to_string_lossy()));
    args.push("-Djna.nosys=true".to_string());
    args.push("-Djna.noclasspath=false".to_string());
    
    // LWJGL configuration to use pre-extracted natives
    args.push(format!("-Dorg.lwjgl.librarypath={}", natives_path.to_string_lossy()));
    args.push("-Dorg.lwjgl.system.SharedLibraryExtractDirectory=false".to_string());
    args.push("-Dorg.lwjgl.system.SharedLibraryExtractPath=false".to_string());
    
    // More aggressive LWJGL configuration to force using system libraries
    args.push("-Dorg.lwjgl.util.NoChecks=true".to_string());
    args.push("-Dorg.lwjgl.system.SharedLibraryLoader.load=false".to_string());
    
    // Additional LWJGL debugging (remove after testing)
    args.push("-Dorg.lwjgl.util.Debug=true".to_string());
    args.push("-Dorg.lwjgl.util.DebugLoader=true".to_string());

    // Build classpath: version JAR + all library JARs
    let mut classpath_entries = Vec::new();
    // Add all library JARs recursively
    if let Ok(entries) = std::fs::read_dir(&libraries_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let walker = match walkdir::WalkDir::new(&path).into_iter().collect::<Vec<_>>() {
                    v => v,
                };
                for file in walker {
                    if let Ok(file) = file {
                        if let Some(ext) = file.path().extension() {
                            if ext == "jar" {
                                classpath_entries.push(file.path().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            } else if let Some(ext) = path.extension() {
                if ext == "jar" {
                    classpath_entries.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    // Add the version JAR at the beginning (important for Minecraft)
    classpath_entries.insert(0, version_jar.to_string_lossy().to_string());
    
    // Sort classpath to ensure JNA libraries are loaded early
    let mut regular_entries = Vec::new();
    let mut jna_entries = Vec::new();
    
    for entry in classpath_entries {
        if entry.contains("jna") {
            jna_entries.push(entry);
        } else {
            regular_entries.push(entry);
        }
    }
    
    // Put JNA libraries first, then everything else
    let mut final_classpath = jna_entries;
    final_classpath.extend(regular_entries);
    
    // Join with semicolon for Windows
    let classpath = final_classpath.join(";");
    
    println!("JNA libraries in classpath:");
    for entry in &final_classpath {
        if entry.contains("jna") {
            println!("  {}", entry);
        }
    }
    // Minecraft launch arguments
    args.extend([
        "-cp".to_string(),
        classpath,
        "net.minecraft.client.main.Main".to_string(),
        "--version".to_string(),
        version.clone(),
        "--gameDir".to_string(),
        instance_path.clone(),
        "--assetsDir".to_string(),
        format!("{}\\assets", instance_path),
        "--assetIndex".to_string(),
        version.clone(),
    ]);
    
    // Add authentication arguments if we have an active account
    if let Some(_token) = &auth_token {
        // Try to get account details for username/UUID
        match crate::auth::get_stored_accounts().await {
            Ok(accounts) if !accounts.is_empty() => {
                let account = &accounts[0]; // Use first account
                args.extend([
                    "--username".to_string(),
                    account.username.clone(),
                    "--uuid".to_string(),
                    account.uuid.clone(),
                    "--accessToken".to_string(),
                    account.access_token.clone(),
                    "--userType".to_string(),
                    "msa".to_string(),
                ]);
                println!("Added authentication for user: {}", account.username);
            }
            _ => {
                // Fallback to offline mode
                args.extend([
                    "--username".to_string(),
                    "Player".to_string(),
                    "--uuid".to_string(),
                    "00000000-0000-0000-0000-000000000000".to_string(),
                    "--accessToken".to_string(),
                    "0".to_string(),
                    "--userType".to_string(),
                    "legacy".to_string(),
                ]);
                println!("Using offline mode authentication");
            }
        }
    } else {
        // Offline mode
        args.extend([
            "--username".to_string(),
            "Player".to_string(),
            "--uuid".to_string(),
            "00000000-0000-0000-0000-000000000000".to_string(),
            "--accessToken".to_string(),
            "0".to_string(),
            "--userType".to_string(),
            "legacy".to_string(),
        ]);
        println!("Using offline mode authentication (no token)");
    }

    println!("Launching Minecraft with command: {} {}", java_path, args.join(" "));
    println!("Working directory: {}", instance_path);
    println!("Classpath entries: {}", final_classpath.len());
    
    // Check critical directories
    let assets_dir = std::path::Path::new(&instance_path).join("assets");
    let natives_dir = std::path::Path::new(&instance_path).join("natives");
    println!("Assets directory exists: {} ({})", assets_dir.exists(), assets_dir.display());
    println!("Natives directory exists: {} ({})", natives_dir.exists(), natives_dir.display());
    
    if !assets_dir.exists() {
        println!("WARNING: Assets directory missing - this will likely cause Minecraft to fail!");
    }
    if !natives_dir.exists() {
        println!("WARNING: Natives directory missing - this will likely cause graphics issues!");
        println!("Attempting to extract native libraries...");
        
        // Create natives directory
        if let Err(e) = std::fs::create_dir_all(&natives_dir) {
            println!("Failed to create natives directory: {}", e);
        } else {
            // Extract natives from JAR files
            extract_native_libraries(&instance_path, &natives_dir).await?;
        }
    }

    let mut child = Command::new(&java_path)
        .args(&args)
        .current_dir(&instance_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;

    // Log process ID for debugging
    println!("Minecraft process started with PID: {}", child.id().unwrap_or(0));

    // Give the process a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Check if process started successfully
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process already exited
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            
            println!("Process exited with status: {:?}", status);
            
            if let Some(mut stdout) = stdout {
                let mut output = String::new();
                let _ = stdout.read_to_string(&mut output).await;
                println!("Minecraft stdout: {}", output);
            }
            
            if let Some(mut stderr) = stderr {
                let mut error = String::new();
                let _ = stderr.read_to_string(&mut error).await;
                println!("Minecraft stderr: {}", error);
            }
            
            return Err(format!("Minecraft process exited with status: {:?}. Check logs above for details.", status));
        }
        Ok(None) => {
            // Process is still running
            println!("Minecraft process is still running after 2 seconds - launch appears successful");
            
            // Let's monitor it for a bit longer to see what happens
            for i in 1..=10 {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("Process exited after {} seconds with status: {:?}", i + 2, status);
                        
                        // Try to read any output
                        if let Some(mut stdout) = child.stdout.take() {
                            let mut output = String::new();
                            let _ = stdout.read_to_string(&mut output).await;
                            if !output.trim().is_empty() {
                                println!("Minecraft stdout (after exit): {}", output);
                            }
                        }
                        
                        if let Some(mut stderr) = child.stderr.take() {
                            let mut error = String::new();
                            let _ = stderr.read_to_string(&mut error).await;
                            if !error.trim().is_empty() {
                                println!("Minecraft stderr (after exit): {}", error);
                            }
                        }
                        
                        return Err(format!("Minecraft exited after {} seconds with status: {:?}", i + 2, status));
                    }
                    Ok(None) => {
                        if i == 5 {
                            println!("Process still running after {} seconds...", i + 2);
                        }
                        if i == 10 {
                            println!("Process has been running for {} seconds. Minecraft should be visible now.", i + 2);
                            println!("If you don't see Minecraft window, check:");
                            println!("1. Task Manager for java.exe process");
                            println!("2. Assets directory: {}\\assets", instance_path);
                            println!("3. Natives directory: {}\\natives", instance_path);
                        }
                    }
                    Err(e) => {
                        println!("Error checking process after {} seconds: {}", i + 2, e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to check process status: {}", e));
        }
    }

    Ok(())
}

async fn extract_native_libraries(instance_path: &str, natives_dir: &std::path::Path) -> Result<(), String> {
    
    let libraries_dir = std::path::Path::new(instance_path).join("libraries");
    println!("Extracting native libraries from: {}", libraries_dir.display());
    
    let mut extracted_count = 0;
    
    // Find all native JAR files (containing -natives- in filename)
    if let Ok(entries) = std::fs::read_dir(&libraries_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively search in subdirectories
                extract_natives_from_directory(&path, natives_dir, &mut extracted_count)?;
            }
        }
    }
    
    println!("Extracted {} native library files", extracted_count);
    
    if extracted_count == 0 {
        return Err("No native libraries found to extract".to_string());
    }
    
    Ok(())
}

fn extract_natives_from_directory(
    dir: &std::path::Path, 
    natives_dir: &std::path::Path, 
    extracted_count: &mut i32
) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recurse into subdirectories
                extract_natives_from_directory(&path, natives_dir, extracted_count)?;
            } else if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                
                // Check if this is a natives JAR file
                if filename_str.ends_with(".jar") && 
                   (filename_str.contains("-natives-windows") || 
                    filename_str.contains("-natives-")) {
                    
                    println!("Extracting natives from: {}", filename_str);
                    
                    // Extract DLL files from this JAR
                    match extract_dll_from_jar(&path, natives_dir) {
                        Ok(count) => {
                            *extracted_count += count;
                        }
                        Err(e) => {
                            println!("Warning: Failed to extract from {}: {}", filename_str, e);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_dll_from_jar(jar_path: &std::path::Path, natives_dir: &std::path::Path) -> Result<i32, String> {
    
    let file = std::fs::File::open(jar_path)
        .map_err(|e| format!("Failed to open JAR: {}", e))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP: {}", e))?;
    
    let mut extracted_count = 0;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;
        
        let file_name = file.name();
        
        // Extract DLL, SO, and DYLIB files
        if file_name.ends_with(".dll") || 
           file_name.ends_with(".so") || 
           file_name.ends_with(".dylib") {
            
            // Get just the filename without path
            if let Some(filename) = std::path::Path::new(file_name).file_name() {
                let output_path = natives_dir.join(filename);
                
                println!("  Extracting: {} -> {}", file_name, output_path.display());
                
                let mut output_file = std::fs::File::create(&output_path)
                    .map_err(|e| format!("Failed to create output file: {}", e))?;
                
                std::io::copy(&mut file, &mut output_file)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                
                extracted_count += 1;
            }
        }
    }
    
    Ok(extracted_count)
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