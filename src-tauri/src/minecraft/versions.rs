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

    // Check direct path first
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
                
                if potential_java.exists() {
                    return Ok(potential_java.to_string_lossy().to_string());
                }
            }
        }
    }

    Err(format!(
        "Java {} not found in ChaiLauncher's Java directory: {}\nPlease install Java {} through ChaiLauncher", 
        java_version, java_dir.display(), java_version
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

/// Load version manifest (compatibility function)
pub async fn load_version_manifest(
    _game_dir: &std::path::Path,
    _version: &str,
) -> Result<Option<serde_json::Value>, String> {
    // For now, return None to indicate we should use MCVM's version management
    // This can be enhanced later if needed
    Ok(None)
}