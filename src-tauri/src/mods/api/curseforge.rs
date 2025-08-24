use async_trait::async_trait;
use reqwest::Client;
use crate::mods::types::*;
use crate::mods::api::ModApi;
use std::path::Path;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde_json;

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
            api_key: Self::get_api_key_from_config(),
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key: Some(api_key),
        }
    }

    fn get_api_key_from_config() -> Option<String> {
        // Try to get API key from environment variable first
        if let Ok(key) = std::env::var("CURSEFORGE_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }
        
        // Try to get from config file
        if let Ok(launcher_dir) = crate::storage::get_launcher_dir() {
            let config_path = launcher_dir.join("config").join("curseforge.json");
            if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_content) {
                    if let Some(api_key) = config.get("api_key").and_then(|k| k.as_str()) {
                        if !api_key.is_empty() {
                            return Some(api_key.to_string());
                        }
                    }
                }
            }
        }
        
        None
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
        // For now, return a few sample popular mods to demonstrate functionality
        // This would be replaced with actual CurseForge API calls once API key is configured
        if _query.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(vec![
                ModInfo {
                    id: "jei".to_string(),
                    name: "Just Enough Items (JEI)".to_string(),
                    description: "JEI is an item and recipe viewing mod for Minecraft, built from the ground up for stability and performance.".to_string(),
                    author: "mezz".to_string(),
                    version: "15.2.0.27".to_string(),
                    game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string(), "1.18.2".to_string()],
                    loaders: vec!["forge".to_string(), "neoforge".to_string()],
                    downloads: 500000000,
                    icon_url: None,
                    website_url: Some("https://www.curseforge.com/minecraft/mc-mods/jei".to_string()),
                    source_url: None,
                    license: Some("MIT".to_string()),
                    categories: vec!["Utility".to_string(), "API".to_string()],
                    side: ModSide::Both,
                    source: ModSource::CurseForge,
                    featured: true,
                    date_created: chrono::Utc::now() - chrono::Duration::days(365),
                    date_updated: chrono::Utc::now() - chrono::Duration::days(30),
                },
                ModInfo {
                    id: "waystones".to_string(),
                    name: "Waystones".to_string(),
                    description: "Teleport back to activated waystones. For Survival, Adventure or Creative!".to_string(),
                    author: "BlayTheNinth".to_string(),
                    version: "14.1.3".to_string(),
                    game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string()],
                    loaders: vec!["forge".to_string(), "fabric".to_string(), "neoforge".to_string()],
                    downloads: 150000000,
                    icon_url: None,
                    website_url: Some("https://www.curseforge.com/minecraft/mc-mods/waystones".to_string()),
                    source_url: None,
                    license: Some("MIT".to_string()),
                    categories: vec!["Adventure".to_string(), "Utility".to_string()],
                    side: ModSide::Both,
                    source: ModSource::CurseForge,
                    featured: true,
                    date_created: chrono::Utc::now() - chrono::Duration::days(800),
                    date_updated: chrono::Utc::now() - chrono::Duration::days(15),
                },
            ])
        }
    }

    async fn get_mod_details(&self, mod_id: &str) -> Result<ModInfo, ModError> {
        // For basic functionality, return details based on the mod_id
        // This would be replaced with actual API calls when API key is available
        match mod_id {
            "jei" => Ok(ModInfo {
                id: "jei".to_string(),
                name: "Just Enough Items (JEI)".to_string(),
                description: "JEI is an item and recipe viewing mod for Minecraft, built from the ground up for stability and performance.".to_string(),
                author: "mezz".to_string(),
                version: "15.2.0.27".to_string(),
                game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string(), "1.18.2".to_string()],
                loaders: vec!["forge".to_string(), "neoforge".to_string()],
                downloads: 500000000,
                icon_url: None,
                website_url: Some("https://www.curseforge.com/minecraft/mc-mods/jei".to_string()),
                source_url: None,
                license: Some("MIT".to_string()),
                categories: vec!["Utility".to_string(), "API".to_string()],
                side: ModSide::Both,
                source: ModSource::CurseForge,
                featured: true,
                date_created: chrono::Utc::now() - chrono::Duration::days(365),
                date_updated: chrono::Utc::now() - chrono::Duration::days(30),
            }),
            "waystones" => Ok(ModInfo {
                id: "waystones".to_string(),
                name: "Waystones".to_string(),
                description: "Teleport back to activated waystones. For Survival, Adventure or Creative!".to_string(),
                author: "BlayTheNinth".to_string(),
                version: "14.1.3".to_string(),
                game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string()],
                loaders: vec!["forge".to_string(), "fabric".to_string(), "neoforge".to_string()],
                downloads: 150000000,
                icon_url: None,
                website_url: Some("https://www.curseforge.com/minecraft/mc-mods/waystones".to_string()),
                source_url: None,
                license: Some("MIT".to_string()),
                categories: vec!["Adventure".to_string(), "Utility".to_string()],
                side: ModSide::Both,
                source: ModSource::CurseForge,
                featured: true,
                date_created: chrono::Utc::now() - chrono::Duration::days(800),
                date_updated: chrono::Utc::now() - chrono::Duration::days(15),
            }),
            _ => Err(ModError::NotFound(format!("Mod {} not found in CurseForge stub", mod_id)))
        }
    }

    async fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, ModError> {
        // Return mock mod files for basic functionality
        match mod_id {
            "jei" | "waystones" => Ok(vec![
                ModFile {
                    id: format!("{}-latest", mod_id),
                    mod_id: mod_id.to_string(),
                    filename: format!("{}-1.20.1-latest.jar", mod_id),
                    display_name: format!("{} Latest", mod_id),
                    version: "latest".to_string(),
                    size: 5000000, // 5MB
                    download_url: format!("https://example.com/mods/{}/latest.jar", mod_id),
                    hashes: std::collections::HashMap::new(),
                    dependencies: vec![],
                    game_versions: vec!["1.20.1".to_string()],
                    loaders: vec!["forge".to_string(), "fabric".to_string()],
                    release_type: ReleaseType::Release,
                    date_published: chrono::Utc::now() - chrono::Duration::days(30),
                    primary: true,
                },
            ]),
            _ => Err(ModError::NotFound(format!("No files found for mod {}", mod_id)))
        }
    }

    async fn get_mod_file(&self, mod_id: &str, file_id: &str) -> Result<ModFile, ModError> {
        let files = self.get_mod_files(mod_id).await?;
        files.into_iter()
            .find(|f| f.id == file_id)
            .ok_or_else(|| ModError::NotFound(format!("File {} not found for mod {}", file_id, mod_id)))
    }

    async fn download_mod_file(&self, file: &ModFile, path: &Path, progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>) -> Result<(), ModError> {
        // For basic functionality, create a dummy mod file
        // This would be replaced with actual download logic when API is implemented
        println!("Mock downloading mod file: {} to {:?}", file.filename, path);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Simulate download progress
        let total_size = file.size;
        for i in 0..=10 {
            let downloaded = (total_size * i) / 10;
            progress_callback(downloaded, total_size);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Create a dummy jar file with some content
        let dummy_content = format!("# Dummy mod file for {}\n# This is a placeholder until real downloading is implemented\n", file.mod_id);
        tokio::fs::write(path, dummy_content.as_bytes()).await?;
        
        println!("Mock download completed: {}", file.filename);
        Ok(())
    }

    async fn get_featured_mods(
        &self,
        _game_version: Option<&str>,
        _mod_loader: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        // Return some popular/featured mods as examples
        Ok(vec![
            ModInfo {
                id: "jei".to_string(),
                name: "Just Enough Items (JEI)".to_string(),
                description: "JEI is an item and recipe viewing mod for Minecraft, built from the ground up for stability and performance.".to_string(),
                author: "mezz".to_string(),
                version: "15.2.0.27".to_string(),
                game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string(), "1.18.2".to_string()],
                loaders: vec!["forge".to_string(), "neoforge".to_string()],
                downloads: 500000000,
                icon_url: None,
                website_url: Some("https://www.curseforge.com/minecraft/mc-mods/jei".to_string()),
                source_url: None,
                license: Some("MIT".to_string()),
                categories: vec!["Utility".to_string(), "API".to_string()],
                side: ModSide::Both,
                source: ModSource::CurseForge,
                featured: true,
                date_created: chrono::Utc::now() - chrono::Duration::days(365),
                date_updated: chrono::Utc::now() - chrono::Duration::days(30),
            },
            ModInfo {
                id: "optifine".to_string(),
                name: "OptiFine".to_string(),
                description: "OptiFine is a Minecraft optimization mod. It allows Minecraft to run faster and look better with full support for HD textures and many configuration options.".to_string(),
                author: "sp614x".to_string(),
                version: "HD_U_I5".to_string(),
                game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string(), "1.18.2".to_string()],
                loaders: vec!["optifine".to_string()],
                downloads: 800000000,
                icon_url: None,
                website_url: Some("https://optifine.net/".to_string()),
                source_url: None,
                license: None,
                categories: vec!["Performance".to_string(), "Client".to_string()],
                side: ModSide::Client,
                source: ModSource::Direct("https://optifine.net/".to_string()),
                featured: true,
                date_created: chrono::Utc::now() - chrono::Duration::days(2000),
                date_updated: chrono::Utc::now() - chrono::Duration::days(45),
            },
            ModInfo {
                id: "iron-chests".to_string(),
                name: "Iron Chests".to_string(),
                description: "A variety of new chests with greater storage capacity than the standard wooden chest.".to_string(),
                author: "ProgWML6".to_string(),
                version: "14.4.4".to_string(),
                game_versions: vec!["1.20.1".to_string(), "1.19.4".to_string()],
                loaders: vec!["forge".to_string(), "fabric".to_string(), "neoforge".to_string()],
                downloads: 125000000,
                icon_url: None,
                website_url: Some("https://www.curseforge.com/minecraft/mc-mods/iron-chests".to_string()),
                source_url: None,
                license: Some("MIT".to_string()),
                categories: vec!["Storage".to_string(), "Utility".to_string()],
                side: ModSide::Both,
                source: ModSource::CurseForge,
                featured: true,
                date_created: chrono::Utc::now() - chrono::Duration::days(1200),
                date_updated: chrono::Utc::now() - chrono::Duration::days(20),
            },
        ])
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