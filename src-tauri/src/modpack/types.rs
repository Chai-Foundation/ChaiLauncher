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