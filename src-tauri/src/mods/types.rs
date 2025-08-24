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
    pub async fn is_compatible(&self, _mc_version: &str) -> bool {
        // TODO: Implement actual compatibility checking
        // This would check against known compatibility matrices
        true
    }
    
    /// Get available versions for a loader type and MC version
    pub async fn get_available_versions(_loader_name: &str, _mc_version: &str) -> Vec<String> {
        // TODO: Implement fetching available loader versions
        // This would query the respective APIs for available versions
        vec![]
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