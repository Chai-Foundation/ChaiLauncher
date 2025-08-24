use async_trait::async_trait;
use reqwest::Client;
use crate::mods::types::*;
use crate::mods::api::ModApi;
use std::path::Path;

/// CurseForge API client implementation
/// Note: CurseForge requires an API key for full functionality
#[derive(Debug)]
pub struct CurseForgeApi {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl CurseForgeApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key: None, // TODO: Add API key configuration
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key: Some(api_key),
        }
    }

    fn _has_api_key(&self) -> bool {
        self.api_key.is_some()
    }
}

#[async_trait]
impl ModApi for CurseForgeApi {
    async fn search_mods(
        &self,
        _query: &str,
        _game_version: Option<&str>,
        _mod_loader: Option<&str>,
        _limit: u32,
        _offset: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        // CurseForge API requires an API key
        // For now, return empty list until API key is configured
        Ok(Vec::new())
    }

    async fn get_mod_details(&self, _mod_id: &str) -> Result<ModInfo, ModError> {
        Err(ModError::LoaderNotSupported("CurseForge API requires API key (not yet implemented)".to_string()))
    }

    async fn get_mod_files(&self, _mod_id: &str) -> Result<Vec<ModFile>, ModError> {
        Err(ModError::LoaderNotSupported("CurseForge API requires API key (not yet implemented)".to_string()))
    }

    async fn get_mod_file(&self, _mod_id: &str, _file_id: &str) -> Result<ModFile, ModError> {
        Err(ModError::LoaderNotSupported("CurseForge API requires API key (not yet implemented)".to_string()))
    }

    async fn download_mod_file(&self, _file: &ModFile, _path: &Path, _progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>) -> Result<(), ModError> {
        Err(ModError::LoaderNotSupported("CurseForge API requires API key (not yet implemented)".to_string()))
    }

    async fn get_featured_mods(
        &self,
        _game_version: Option<&str>,
        _mod_loader: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        Ok(Vec::new())
    }

    async fn get_categories(&self) -> Result<Vec<String>, ModError> {
        // Return some common categories for now
        Ok(vec![
            "Technology".to_string(),
            "Magic".to_string(),
            "Adventure".to_string(),
            "Decoration".to_string(),
            "Utility".to_string(),
            "Library".to_string(),
        ])
    }

    async fn check_updates(&self, _installed_mod: &InstalledMod) -> Result<Option<ModFile>, ModError> {
        Ok(None)
    }
}