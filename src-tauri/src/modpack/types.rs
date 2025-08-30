use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthPack {
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u32,
    pub icon_url: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: u32,
    pub version_type: String,
    pub status: String,
    pub requested_status: Option<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackCreationRequest {
    pub instance_id: String,
    pub instance_path: String,
    pub metadata: ModpackMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub minecraft_version: String,
    pub tags: Vec<String>,
    pub icon_path: Option<String>,
    pub include_user_data: bool,
    pub include_resource_packs: bool,
    pub include_shader_packs: bool,
    pub include_config: bool,
    pub include_saves: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackCreationProgress {
    pub instance_id: String,
    pub progress: f64,
    pub stage: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackInstallProgress {
    pub instance_dir: String,
    pub progress: f64,
    pub stage: String,
}

pub struct ModpackInstaller {
    pub client: Client,
    pub instance_dir: PathBuf,
}

impl ModpackInstaller {
    pub fn new(instance_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            instance_dir,
        }
    }
}

pub struct ModpackCreator;

impl ModpackCreator {
    pub fn new() -> Self {
        Self
    }
}

/// Proper .mrpack format structures
#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackFile {
    pub path: String,
    pub hashes: MrpackHashes,
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackHashes {
    pub sha1: String,
    pub sha512: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MrpackEnv {
    pub client: Option<String>,
    pub server: Option<String>,
}