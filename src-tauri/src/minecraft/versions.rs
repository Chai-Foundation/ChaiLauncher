//! Version Management using MCVM
//! 
//! This module handles Minecraft version detection, installation, and management
//! using MCVM's version system while maintaining ChaiLauncher's own Java management.

use serde::{Deserialize, Serialize};

/// Minecraft version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub r#type: String,
    pub release_time: String,
    pub compatible_java_versions: Vec<u32>,
}

/// Version manifest structure
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// Get required Java version for a Minecraft version
pub fn get_required_java_version(version: &str) -> u32 {
    // Use version parsing logic to determine Java requirements
    if version_compare(version, "1.21") >= 0 {
        21 // Java 21+ required for 1.21+
    } else if version_compare(version, "1.17") >= 0 {
        17 // Java 17+ required for 1.17+
    } else if version_compare(version, "1.12") >= 0 {
        8 // Java 8+ required for 1.12+
    } else {
        8 // Java 8 for older versions
    }
}

/// Get ChaiLauncher's own Java executable path for a version
/// This maintains ChaiLauncher's independent Java management
pub async fn get_java_for_version(java_version: u32) -> Result<String, String> {
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java").join(format!("java{}", java_version));
    
    let java_exe = if cfg!(target_os = "windows") {
        java_dir.join("bin").join("java.exe")
    } else {
        java_dir.join("bin").join("java")
    };

    // Track paths checked for debugging
    let mut checked_paths = Vec::new();
    
    // Check direct path first
    checked_paths.push(java_exe.to_string_lossy().to_string());
    if java_exe.exists() {
        return Ok(java_exe.to_string_lossy().to_string());
    }

    // Look for nested JDK directories (from ChaiLauncher's own Java installations)
    if let Ok(entries) = std::fs::read_dir(&java_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let potential_java = if cfg!(target_os = "windows") {
                    entry.path().join("bin").join("java.exe")
                } else {
                    entry.path().join("bin").join("java")
                };
                
                checked_paths.push(potential_java.to_string_lossy().to_string());
                if potential_java.exists() {
                    return Ok(potential_java.to_string_lossy().to_string());
                }

                // Check for macOS Contents/Home/bin/java path
                if cfg!(target_os = "macos") {
                    let macos_java_path = entry.path().join("Contents").join("Home").join("bin").join("java");
                    
                    checked_paths.push(macos_java_path.to_string_lossy().to_string());
                    if macos_java_path.exists() {
                        return Ok(macos_java_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // Enhanced error message with debugging information
    let os_info = if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Linux"
    };
    
    let java_dir_exists = java_dir.exists();
    let launcher_dir_exists = launcher_dir.exists();
    
    Err(format!(
        "Java {} not found in ChaiLauncher's Java directory.\n\
        Platform: {}\n\
        Launcher directory: {} (exists: {})\n\
        Java directory: {} (exists: {})\n\
        Paths checked: {}\n\
        \n\
        Please install Java {} through ChaiLauncher or check if the Java installation is corrupted.", 
        java_version,
        os_info,
        launcher_dir.display(), launcher_dir_exists,
        java_dir.display(), java_dir_exists,
        checked_paths.join(", "),
        java_version
    ))
}

/// Check if ChaiLauncher's own Java version is installed
pub async fn is_java_version_installed(java_version: u32) -> bool {
    get_java_for_version(java_version).await.is_ok()
}

/// Compare two version strings
pub fn version_compare(a: &str, b: &str) -> i32 {
    let a_parts: Vec<i32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<i32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    
    let max_len = a_parts.len().max(b_parts.len());
    
    for i in 0..max_len {
        let a_part = a_parts.get(i).unwrap_or(&0);
        let b_part = b_parts.get(i).unwrap_or(&0);
        
        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Less => return -1,
        }
    }
    
    0
}

/// Load version manifest from the versions directory
pub async fn load_version_manifest(
    game_dir: &std::path::Path,
    version: &str,
) -> Result<Option<serde_json::Value>, String> {
    use tokio::fs;
    
    // Look for version JSON in versions/{version}/{version}.json
    let version_file = game_dir.join("versions").join(version).join(format!("{}.json", version));
    
    println!("🔍 Looking for version manifest at: {}", version_file.display());
    
    if !version_file.exists() {
        println!("❌ Version manifest file does not exist");
        return Ok(None);
    }
    
    match fs::read_to_string(&version_file).await {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(json) => {
                    println!("✅ Version manifest loaded and parsed successfully");
                    Ok(Some(json))
                },
                Err(e) => {
                    println!("❌ Failed to parse version manifest JSON: {}", e);
                    Err(format!("Failed to parse version manifest: {}", e))
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to read version manifest file: {}", e);
            Err(format!("Failed to read version manifest: {}", e))
        }
    }
}