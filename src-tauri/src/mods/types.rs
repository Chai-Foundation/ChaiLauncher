use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Represents a mod from any API source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u32,
    pub icon_url: Option<String>,
    pub website_url: Option<String>,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub categories: Vec<String>,
    pub side: ModSide,
    pub source: ModSource,
    pub featured: bool,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
}

/// Represents a specific file/version of a mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub id: String,
    pub mod_id: String,
    pub filename: String,
    pub display_name: String,
    pub version: String,
    pub size: u64,
    pub download_url: String,
    pub hashes: HashMap<String, String>,
    pub dependencies: Vec<ModDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub release_type: ReleaseType,
    pub date_published: DateTime<Utc>,
    pub primary: bool,
}

/// Represents a mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType,
}

/// Represents an installed mod in an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub mod_info: ModInfo,
    pub installed_file: ModFile,
    pub install_path: PathBuf,
    pub enabled: bool,
    pub install_date: DateTime<Utc>,
    pub update_available: Option<ModFile>,
}

/// Supported mod loaders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModLoader {
    Forge(String),      // version
    Fabric(String),     // loader version
    Quilt(String),      // loader version
    NeoForge(String),   // version
    ModLoader(String),  // legacy version
    Rift(String),       // legacy 1.13.x version
}

/// Where the mod can be installed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModSide {
    Client,
    Server,
    Both,
    Unknown,
}

/// Source of the mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModSource {
    CurseForge,
    Modrinth,
    GitHub,
    Direct(String), // URL
    Local,
}

/// Type of mod dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

/// Release type for mod files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReleaseType {
    Release,
    Beta,
    Alpha,
}

/// Error types for mod operations
#[derive(Debug, thiserror::Error)]
pub enum ModError {
    #[error("API error: {0}")]
    Api(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Mod not found: {0}")]
    NotFound(String),
    #[error("Dependency conflict: {0}")]
    DependencyConflict(String),
    #[error("Version incompatible: {0}")]
    VersionIncompatible(String),
    #[error("Mod loader not supported: {0}")]
    LoaderNotSupported(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Invalid mod file: {0}")]
    InvalidFile(String),
}

impl ModLoader {
    pub fn name(&self) -> &str {
        match self {
            ModLoader::Forge(_) => "forge",
            ModLoader::Fabric(_) => "fabric",
            ModLoader::Quilt(_) => "quilt",
            ModLoader::NeoForge(_) => "neoforge",
            ModLoader::ModLoader(_) => "modloader",
            ModLoader::Rift(_) => "rift",
        }
    }
    
    pub fn version(&self) -> &str {
        match self {
            ModLoader::Forge(v) => v,
            ModLoader::Fabric(v) => v,
            ModLoader::Quilt(v) => v,
            ModLoader::NeoForge(v) => v,
            ModLoader::ModLoader(v) => v,
            ModLoader::Rift(v) => v,
        }
    }
    
    /// Check if this loader is compatible with a game version
    pub async fn is_compatible(&self, mc_version: &str) -> bool {
        // Implement basic compatibility checking based on known version ranges
        match self {
            ModLoader::Forge(_) => {
                // Forge supports most MC versions, but check basic compatibility
                let mc_parts: Vec<&str> = mc_version.split('.').collect();
                if mc_parts.len() >= 2 {
                    let major: i32 = mc_parts[1].parse().unwrap_or(0);
                    major >= 12 // Forge generally supports 1.12+ 
                } else {
                    false
                }
            },
            ModLoader::Fabric(_) => {
                // Fabric supports newer versions well
                let mc_parts: Vec<&str> = mc_version.split('.').collect();
                if mc_parts.len() >= 2 {
                    let major: i32 = mc_parts[1].parse().unwrap_or(0);
                    major >= 14 // Fabric generally supports 1.14+
                } else {
                    false
                }
            },
            ModLoader::Quilt(_) => {
                // Quilt is Fabric-compatible and supports similar versions
                let mc_parts: Vec<&str> = mc_version.split('.').collect();
                if mc_parts.len() >= 2 {
                    let major: i32 = mc_parts[1].parse().unwrap_or(0);
                    major >= 14 // Quilt generally supports 1.14+
                } else {
                    false
                }
            },
            ModLoader::NeoForge(_) => {
                // NeoForge is for newer versions (1.20+)
                let mc_parts: Vec<&str> = mc_version.split('.').collect();
                if mc_parts.len() >= 2 {
                    let major: i32 = mc_parts[1].parse().unwrap_or(0);
                    major >= 20 // NeoForge generally supports 1.20+
                } else {
                    false
                }
            },
            ModLoader::ModLoader(_) => {
                // Legacy ModLoader, very old versions
                let mc_parts: Vec<&str> = mc_version.split('.').collect();
                if mc_parts.len() >= 2 {
                    let major: i32 = mc_parts[1].parse().unwrap_or(0);
                    major <= 12 // ModLoader was for very old versions
                } else {
                    false
                }
            },
            ModLoader::Rift(_) => {
                // Rift was specifically for 1.13.x
                mc_version.starts_with("1.13")
            },
        }
    }
    
    /// Get available versions for a loader type and MC version
    pub async fn get_available_versions(loader_name: &str, mc_version: &str) -> Vec<String> {
        // Use the ModLoaderManager to fetch available versions
        let temp_path = std::env::temp_dir().join("temp_loader_versions");
        let manager = crate::mods::loaders::ModLoaderManager::new(temp_path);
        
        match manager.get_available_versions(loader_name, mc_version).await {
            Ok(versions) => versions,
            Err(_) => {
                // Fallback to basic version lists if API calls fail
                match loader_name.to_lowercase().as_str() {
                    "forge" => vec!["47.2.20".to_string(), "47.2.0".to_string(), "47.1.0".to_string()],
                    "fabric" => vec!["0.15.3".to_string(), "0.14.24".to_string(), "0.14.23".to_string()],
                    "quilt" => vec!["0.21.1".to_string(), "0.21.0".to_string(), "0.20.2".to_string()],
                    "neoforge" => vec!["20.4.195".to_string(), "20.4.109".to_string(), "20.4.108".to_string()],
                    _ => vec![],
                }
            }
        }
    }
}

impl ModSource {
    pub fn api_name(&self) -> &str {
        match self {
            ModSource::CurseForge => "curseforge",
            ModSource::Modrinth => "modrinth",
            ModSource::GitHub => "github",
            ModSource::Direct(_) => "direct",
            ModSource::Local => "local",
        }
    }
}