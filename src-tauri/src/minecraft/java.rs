//! Java Installation Management for ChaiLauncher
//!
//! This module provides all logic for managing Java installations within ChaiLauncher, including downloading, installing, validating, and discovering Java versions.
//!
//! # Features
//! - Downloads Java distributions from Eclipse Temurin
//! - Installs Java in a launcher-managed directory
//! - Tracks installation progress and emits events for UI updates
//! - Extracts and validates Java installations
//! - Supports multiple Java versions (8, 17, 21)
//! - Platform-aware (Windows, macOS, Linux)
//!
//! ChaiLauncher manages its own Java installations, independent of system Java. MCVM integration does not affect this behavior.

use std::path::PathBuf;

/// Represents information about a Java installation managed by ChaiLauncher.
///
/// Fields:
/// - `version`: The major version of Java (e.g., 8, 17, 21).
/// - `path`: The path to the Java executable or installation directory.
/// - `is_installed`: Whether this Java version is installed and available for use.
#[derive(Debug, Clone)]
pub struct JavaInstallation {
    pub version: u32,
    pub path: PathBuf,
    pub is_installed: bool,
}

/// Constructs the download URL for a given Java major version using the Eclipse Temurin API.
///
/// # Arguments
/// * `major_version` - The major Java version to download (e.g., 8, 17, 21).
///
/// # Returns
/// * `Ok(String)` - The download URL for the Java binary.
/// * `Err(String)` - An error message if the OS or architecture is unsupported.
///
/// # Example
/// ```
/// let url = get_java_download_url(17)?;
/// ```
pub fn get_java_download_url(major_version: u32) -> Result<String, String> {
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
    
    // Use Eclipse Temurin API to get latest version
    Ok(format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
        major_version, os, arch
    ))
}

/// Downloads and installs the specified Java version for ChaiLauncher.
///
/// This function:
/// - Checks if the Java version is already installed
/// - Downloads the Java archive from Eclipse Temurin
/// - Tracks download and extraction progress via Tauri events
/// - Extracts the archive and locates the Java executable
///
/// # Arguments
/// * `major_version` - The major Java version to install (e.g., 8, 17, 21)
/// * `app_handle` - Tauri AppHandle for emitting progress events
///
/// # Returns
/// * `Ok(String)` - Path to the installed Java executable
/// * `Err(String)` - Error message if installation fails
pub async fn download_and_install_java(
    major_version: u32,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    use std::fs;
    
    println!("🚀 Starting Java {} installation...", major_version);
    
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java").join(format!("java{}", major_version));
    
    // Check if already installed
    let java_exe = if cfg!(target_os = "windows") {
        java_dir.join("bin").join("java.exe")
    } else {
        java_dir.join("bin").join("java")
    };
    
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
    download_file_with_progress(&download_url, &temp_file, app_handle).await
        .map_err(|e| format!("Failed to download Java {}: {}", major_version, e))?;
    
    // Emit extraction progress
    let _ = app_handle.emit("java_install_progress", serde_json::json!({
        "stage": "Extracting Java...",
        "progress": 85
    }));
    
    println!("📦 Extracting Java {}...", major_version);
    
    // Extract Java
    extract_java_archive(&temp_file, &java_dir, app_handle).await
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

/// Downloads a file from the given URL to the specified destination, emitting progress events for UI updates.
///
/// # Arguments
/// * `url` - The URL to download from
/// * `dest` - Destination path for the downloaded file
/// * `app_handle` - Tauri AppHandle for emitting progress events
///
/// # Returns
/// * `Ok(())` - On successful download
/// * `Err(String)` - On failure
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
        std::io::Write::write_all(&mut file, &chunk)
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

/// Extracts a Java archive (ZIP on Windows, TAR.GZ on Unix) to the specified directory, emitting extraction progress events.
///
/// # Arguments
/// * `archive_path` - Path to the downloaded archive file
/// * `extract_dir` - Directory to extract the contents into
/// * `app_handle` - Tauri AppHandle for emitting progress events
///
/// # Returns
/// * `Ok(())` - On successful extraction
/// * `Err(String)` - On failure
async fn extract_java_archive(
    archive_path: &PathBuf,
    extract_dir: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use zip::ZipArchive;
        use std::fs::File;
        
        let file = File::open(archive_path)
            .map_err(|e| format!("Failed to open archive: {}", e))?;
        
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
        
        let total_files = archive.len();
        for i in 0..total_files {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to extract file {}: {}", i, e))?;
            
            let outpath = match file.enclosed_name() {
                Some(path) => extract_dir.join(path),
                None => continue,
            };
            
            if file.name().ends_with('/') {
                tokio::fs::create_dir_all(&outpath).await
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    tokio::fs::create_dir_all(p).await
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
                
                // Copy file contents to buffer first
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                
                // Write buffer to file
                tokio::fs::write(&outpath, buffer).await
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
            
            // Emit extraction progress
            if i % 50 == 0 {
                let extract_progress = 80 + ((i as f64 / total_files as f64) * 15.0) as u32;
                let _ = app_handle.emit("java_install_progress", serde_json::json!({
                    "stage": format!("Extracting... ({}/{})", i + 1, total_files),
                    "progress": extract_progress
                }));
            }
        }
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
    
    println!("✓ Java extracted successfully");
    Ok(())
}

/// Searches for the Java executable within the extracted Java directory.
///
/// # Arguments
/// * `java_dir` - The root directory of the extracted Java installation
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the Java executable
/// * `Err(String)` - If the executable is not found
fn find_java_executable(java_dir: &PathBuf) -> Result<PathBuf, String> {
    use std::fs;
    
    let java_exe_name = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };
    
    // First try direct path
    let direct_path = java_dir.join("bin").join(java_exe_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }
    
    // Search for nested JDK directories
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

/// Returns a list of Java installations managed by ChaiLauncher, including their version, path, and installation status.
///
/// Checks for Java 8, 17, and 21 by default.
///
/// # Returns
/// * `Ok(Vec<JavaInstallation>)` - List of Java installations
/// * `Err(String)` - On failure
pub async fn get_java_installations() -> Result<Vec<JavaInstallation>, String> {
    let mut installations = Vec::new();
    
    // Check for Java 8, 17, and 21 installations
    for version in [8, 17, 21] {
        let is_installed = super::versions::is_java_version_installed(version).await;
        let path = if is_installed {
            super::versions::get_java_for_version(version).await.ok()
                .map(PathBuf::from).unwrap_or_default()
        } else {
            PathBuf::new()
        };
        
        installations.push(JavaInstallation {
            version,
            path,
            is_installed,
        });
    }
    
    Ok(installations)
}

/// Validates a Java installation by running `java -version` and returning the output.
///
/// # Arguments
/// * `java_path` - Path to the Java executable
///
/// # Returns
/// * `Ok(String)` - Output of `java -version` if successful
/// * `Err(String)` - If validation fails
pub async fn validate_java_installation(java_path: &str) -> Result<String, String> {
    let output = std::process::Command::new(java_path)
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