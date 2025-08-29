use async_trait::async_trait;
use crate::mods::types::*;
use std::path::Path;

pub mod modrinth;
pub mod common;

pub use modrinth::*;

/// Trait that all mod API clients must implement
#[async_trait]
pub trait ModApi: Send + Sync {
    /// Search for mods with the given query
    async fn search_mods(
        &self, 
        query: &str, 
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError>;
    
    /// Get detailed information about a specific mod
    async fn get_mod_details(&self, mod_id: &str) -> Result<ModInfo, ModError>;
    
    /// Get all files/versions for a specific mod
    async fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, ModError>;
    
    /// Get a specific file for a mod
    async fn get_mod_file(&self, mod_id: &str, file_id: &str) -> Result<ModFile, ModError>;
    
    /// Download a mod file to the specified path with progress callback
    async fn download_mod_file(&self, file: &ModFile, path: &Path, progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>) -> Result<(), ModError>;
    
    /// Get featured/popular mods
    async fn get_featured_mods(
        &self,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError>;
    
    /// Get categories available for mods
    async fn get_categories(&self) -> Result<Vec<String>, ModError>;
    
    /// Check if a mod has updates available
    async fn check_updates(&self, installed_mod: &InstalledMod) -> Result<Option<ModFile>, ModError>;
}

/// Enum to hold different API implementations for object safety
#[derive(Debug)]
pub enum ApiClient {
    Modrinth(ModrinthApi),
}

#[async_trait]
impl ModApi for ApiClient {
    async fn search_mods(
        &self, 
        query: &str, 
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.search_mods(query, game_version, mod_loader, limit, offset).await,
        }
    }
    
    async fn get_mod_details(&self, mod_id: &str) -> Result<ModInfo, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.get_mod_details(mod_id).await,
        }
    }
    
    async fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.get_mod_files(mod_id).await,
        }
    }
    
    async fn get_mod_file(&self, mod_id: &str, file_id: &str) -> Result<ModFile, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.get_mod_file(mod_id, file_id).await,
        }
    }
    
    async fn download_mod_file(&self, file: &ModFile, path: &Path, progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>) -> Result<(), ModError> {
        match self {
            ApiClient::Modrinth(api) => api.download_mod_file(file, path, progress_callback).await,
        }
    }
    
    async fn get_featured_mods(
        &self,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.get_featured_mods(game_version, mod_loader, limit, offset).await,
        }
    }
    
    async fn get_categories(&self) -> Result<Vec<String>, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.get_categories().await,
        }
    }
    
    async fn check_updates(&self, installed_mod: &InstalledMod) -> Result<Option<ModFile>, ModError> {
        match self {
            ApiClient::Modrinth(api) => api.check_updates(installed_mod).await,
        }
    }
}

/// Factory for creating API clients
pub struct ApiClientFactory;

impl ApiClientFactory {
    /// Create all available API clients
    pub fn create_all() -> Vec<ApiClient> {
        vec![
            ApiClient::Modrinth(ModrinthApi::new()),
        ]
    }
    
    /// Create a specific API client by name
    pub fn create_by_name(name: &str) -> Option<ApiClient> {
        match name.to_lowercase().as_str() {
            "modrinth" => Some(ApiClient::Modrinth(ModrinthApi::new())),
            _ => None,
        }
    }
}