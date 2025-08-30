use tauri::{command, AppHandle, Emitter};
use std::process::Command;
use std::path::PathBuf;
use tokio::fs;
use reqwest;
use std::io::Write;
use futures::StreamExt;

use crate::storage::StorageManager;

/// Get bundled Java path (defaults to Java 17)
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
pub async fn download_and_install_java(app_handle: AppHandle) -> Result<String, String> {
    download_and_install_java_version(17, app_handle).await
}

/// Download and install Java 8
#[command]
pub async fn download_and_install_java8(app_handle: AppHandle) -> Result<String, String> {
    download_and_install_java_version(8, app_handle).await
}

/// Download and install Java 17
#[command]
pub async fn download_and_install_java17(app_handle: AppHandle) -> Result<String, String> {
    download_and_install_java_version(17, app_handle).await
}

/// Download and install both Java 8 and Java 17
#[command]
pub async fn download_and_install_both_java(app_handle: AppHandle) -> Result<String, String> {
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
pub async fn download_and_install_java_version(major_version: u32, app_handle: AppHandle) -> Result<String, String> {
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
    let api_url = get_java_download_url(major_version)?;
    println!("📥 Fetching download info from: {}", api_url);
    
    // Fetch the API response to get the actual download URL
    let api_response = reqwest::get(&api_url).await
        .map_err(|e| format!("Failed to fetch Java download info: {}", e))?;
    
    if !api_response.status().is_success() {
        return Err(format!("Failed to get Java download info: HTTP {}", api_response.status()));
    }
    
    let releases: serde_json::Value = api_response.json().await
        .map_err(|e| format!("Failed to parse Java API response: {}", e))?;
    
    // Extract the download URL from the response
    let download_url = releases
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|release| release.get("binaries"))
        .and_then(|binaries| binaries.as_array())
        .and_then(|bins| bins.iter().find(|bin| {
            // Prefer tar.gz for Unix/macOS, zip for Windows
            if cfg!(target_os = "windows") {
                bin.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str()).map_or(false, |s| s.ends_with(".zip"))
            } else {
                bin.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str()).map_or(false, |s| s.ends_with(".tar.gz"))
            }
        }))
        .and_then(|bin| bin.get("package"))
        .and_then(|pkg| pkg.get("link"))
        .and_then(|link| link.as_str())
        .ok_or_else(|| format!("No suitable Java {} download found for this platform", major_version))?;
    
    println!("📥 Downloading from: {}", download_url);
    
    // Download Java
    let temp_file = java_dir.join(format!("java{}_temp{}", major_version, 
        if cfg!(target_os = "windows") { ".zip" } else { ".tar.gz" }
    ));
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

/// Analyze Java requirements for an instance by scanning mod JARs
#[command]
pub async fn analyze_instance_java_requirements(instance_id: String) -> Result<crate::minecraft::mod_scanner::InstanceJavaAnalysis, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    let instance = storage.get_instance(&instance_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;
    
    println!("🔍 Analyzing Java requirements for instance '{}'", instance.name);
    
    let scanner = crate::minecraft::mod_scanner::ModJarScanner::new(instance.game_dir.clone());
    let analysis = scanner.analyze_instance_java_requirements().await
        .map_err(|e| format!("Failed to analyze Java requirements: {}", e))?;
    
    println!("✅ Analysis complete: min Java {}, recommended Java {}", 
        analysis.minimum_java_version.unwrap_or(8), 
        analysis.recommended_java_version);
    
    if !analysis.conflicting_requirements.is_empty() {
        println!("⚠️ Conflicts found: {:?}", analysis.conflicting_requirements);
    }
    
    Ok(analysis)
}

/// Get mod Java requirements for a specific mod JAR file
#[command]
pub async fn get_mod_java_requirements(mod_path: String) -> Result<crate::minecraft::mod_scanner::ModJavaRequirement, String> {
    use std::path::PathBuf;
    
    let jar_path = PathBuf::from(mod_path);
    
    if !jar_path.exists() {
        return Err("Mod file does not exist".to_string());
    }
    
    if jar_path.extension().and_then(|s| s.to_str()) != Some("jar") {
        return Err("File is not a JAR file".to_string());
    }
    
    println!("🔍 Scanning mod JAR: {}", jar_path.display());
    
    // Use a dummy scanner instance for single file scanning
    let temp_path = jar_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let scanner = crate::minecraft::mod_scanner::ModJarScanner::new(temp_path);
    
    let requirement = scanner.scan_mod_jar(&jar_path).await
        .map_err(|e| format!("Failed to scan mod JAR: {}", e))?;
    
    println!("✅ Scanned mod: {} requires Java {}", requirement.mod_name, requirement.java_requirement);
    
    Ok(requirement)
}

// Helper functions

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
    
    // Use the assets API instead of binary/latest for better reliability on macOS
    // This will get the latest GA release for the specified version, OS, and architecture
    Ok(format!(
        "https://api.adoptium.net/v3/assets/feature_releases/{}/ga?architecture={}&os={}&image_type=jdk&jvm_impl=hotspot&heap_size=normal&vendor=eclipse",
        major_version, arch, os
    ))
}

/// Download file with progress tracking
async fn download_file_with_progress(
    url: &str,
    dest: &PathBuf,
    app_handle: &AppHandle,
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
async fn extract_java_archive(archive_path: &PathBuf, extract_dir: &PathBuf, app_handle: &AppHandle) -> Result<(), String> {
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
        
        println!("🗜️ Extracting tar.gz archive for macOS/Linux...");
        
        // Ensure the extraction directory exists and has proper permissions
        tokio::fs::create_dir_all(extract_dir).await
            .map_err(|e| format!("Failed to create extraction directory: {}", e))?;
        
        // Use tar command for extraction with verbose output for debugging
        let output = tokio::process::Command::new("tar")
            .args(&[
                "-xzf", 
                &archive_path.to_string_lossy(), 
                "-C", 
                &extract_dir.to_string_lossy(),
                "--strip-components=1"  // Remove the top-level directory from extracted files
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute tar command: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Tar extraction failed: {}\nStdout: {}\nStderr: {}", 
                output.status, stdout, stderr));
        }
        
        println!("✅ Tar extraction completed successfully");
        
        // Set executable permissions on the Java binary for macOS
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::PermissionsExt;
            let java_exe = extract_dir.join("bin").join("java");
            if java_exe.exists() {
                let mut perms = tokio::fs::metadata(&java_exe).await
                    .map_err(|e| format!("Failed to get Java executable metadata: {}", e))?
                    .permissions();
                perms.set_mode(0o755); // rwxr-xr-x
                tokio::fs::set_permissions(&java_exe, perms).await
                    .map_err(|e| format!("Failed to set Java executable permissions: {}", e))?;
                println!("✅ Set executable permissions for Java binary");
            }
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
    
    println!("🔍 Looking for Java executable in: {}", java_dir.display());
    
    // First try direct path (java_dir/bin/java) - this should work with --strip-components=1
    let direct_path = java_dir.join("bin").join(java_exe_name);
    println!("🔍 Checking direct path: {}", direct_path.display());
    if direct_path.exists() {
        println!("✅ Found Java executable at: {}", direct_path.display());
        return Ok(direct_path);
    }
    
    // Search for nested JDK directories as fallback (like jdk-17.0.16+8)
    println!("🔍 Searching for nested JDK directories...");
    let entries = fs::read_dir(java_dir)
        .map_err(|e| format!("Failed to read Java directory: {}", e))?;
        
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let potential_java = path.join("bin").join(java_exe_name);
            println!("🔍 Checking nested path: {}", potential_java.display());
            if potential_java.exists() {
                println!("✅ Found Java executable at: {}", potential_java.display());
                return Ok(potential_java);
            }
        }
    }
    
    // List all contents for debugging
    println!("❌ Java executable not found. Directory contents:");
    if let Ok(entries) = fs::read_dir(java_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  - {}", entry.path().display());
            }
        }
    }
    
    Err(format!("Java executable not found in {}", java_dir.display()))
}