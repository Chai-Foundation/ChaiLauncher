use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use libium::config::filters::Filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String, // "modrinth", "curseforge", "github"
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub version_type: String, // "release", "beta", "alpha"
    pub featured: bool,
    pub date_published: String,
    pub downloads: u64,
    pub changelog: Option<String>,
    pub files: Vec<ModFile>,
    pub dependencies: Vec<ModDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub hashes: std::collections::HashMap<String, String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String, // "required", "optional", "incompatible", "embedded"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub name: String,
    pub slug: Option<String>,
    pub identifier: String, // "platform:id" format
    pub filters: Vec<String>, // Serialized filters
    pub override_filters: bool,
    pub enabled: bool,
    pub install_date: DateTime<Utc>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedMod {
    pub filename: String,
    pub modrinth_project_id: Option<String>,
    pub curseforge_project_id: Option<i32>,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String, // "modrinth", "curseforge"
    pub game_versions: Vec<String>,
    pub mod_loaders: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub version_type: String,
    pub featured: bool,
    pub date_published: String,
    pub downloads: u64,
    pub changelog: Option<String>,
    pub files: Vec<ModpackFile>,
    pub dependencies: Vec<ModpackDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackFile {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub hashes: std::collections::HashMap<String, String>,
    pub primary: bool,
    pub file_type: String, // "mrpack", "zip", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

/// Progress callback for mod/modpack operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: u64,
    pub total: u64,
    pub stage: String,
    pub item_name: Option<String>,
}

/// Request structure for installing a mod
#[derive(Debug, Deserialize)]
pub struct InstallModRequest {
    pub instance_id: String,
    pub mod_id: String,
    pub platform: String, // "modrinth", "curseforge", "github"
    pub version_id: Option<String>,
    pub filters: Option<Vec<String>>, // Serialized filters
}

/// Request structure for installing a modpack
#[derive(Debug, Deserialize)]
pub struct InstallModpackRequest {
    pub instance_name: String,
    pub modpack_id: String,
    pub version_id: String,
    pub platform: String,
    pub game_directory: Option<String>,
    pub install_overrides: bool,
}

/// Configuration for an instance's mod management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceModConfig {
    pub instance_id: String,
    pub profile_name: String,
    pub game_versions: Vec<String>,
    pub mod_loader: String,
    pub output_directory: PathBuf,
    pub auto_update_mods: bool,
    pub check_dependencies: bool,
}