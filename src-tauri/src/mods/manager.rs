use crate::mods::types::*;
use crate::mods::api::{ModApi, ApiClient, ApiClientFactory};
use crate::mods::loaders::ModLoaderManager;
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::fs;
use anyhow::{Result, Context};
use serde_json;

/// Main mod management system for an instance
pub struct ModManager {
    instance_path: PathBuf,
    mods_path: PathBuf,
    loader_manager: ModLoaderManager,
    api_clients: Vec<ApiClient>,
    installed_mods: HashMap<String, InstalledMod>,
}

impl ModManager {
    /// Create a new mod manager for an instance
    pub async fn new(instance_path: PathBuf) -> Result<Self, ModError> {
        let mods_path = instance_path.join("mods");
        
        // Ensure mods directory exists
        fs::create_dir_all(&mods_path).await
            .context("Failed to create mods directory")?;
        
        let loader_manager = ModLoaderManager::new(instance_path.clone());
        let api_clients = ApiClientFactory::create_all();
        let installed_mods = HashMap::new();
        
        let mut manager = Self {
            instance_path,
            mods_path,
            loader_manager,
            api_clients,
            installed_mods,
        };
        
        // Load existing installed mods
        manager.refresh_installed_mods().await?;
        
        Ok(manager)
    }
    
    /// Install a mod by ID from any available API
    pub async fn install_mod<F>(&mut self, mod_id: &str, version_id: Option<&str>, progress_callback: F) -> Result<InstalledMod, ModError>
    where
        F: Fn(u64, u64) + Send + Sync + Clone + 'static,
    {
        // Try to find the mod across all API clients
        let mut mod_info = None;
        let mut selected_client = None;
        
        for client in &self.api_clients {
            match client.get_mod_details(mod_id).await {
                Ok(info) => {
                    mod_info = Some(info);
                    selected_client = Some(client);
                    break;
                }
                Err(_) => continue,
            }
        }
        
        let mod_info = mod_info.ok_or_else(|| ModError::NotFound(mod_id.to_string()))?;
        let client = selected_client.unwrap();
        
        // Get available files
        let files = client.get_mod_files(mod_id).await?;
        
        // Select the appropriate file
        let selected_file = if let Some(version_id) = version_id {
            files.into_iter().find(|f| f.id == version_id)
                .ok_or_else(|| ModError::NotFound(format!("Version {} for mod {}", version_id, mod_id)))?
        } else {
            // Select the latest compatible file
            files.into_iter()
                .filter(|f| f.release_type == ReleaseType::Release)
                .next()
                .ok_or_else(|| ModError::NotFound(format!("No release files found for mod {}", mod_id)))?
        };
        
        // Check dependencies
        self.check_dependencies(&selected_file).await?;
        
        // Download the mod
        let file_path = self.mods_path.join(&selected_file.filename);
        let progress_box: Box<dyn Fn(u64, u64) + Send + Sync> = Box::new(progress_callback);
        client.download_mod_file(&selected_file, &file_path, progress_box).await?;
        
        // Create installed mod record
        let installed_mod = InstalledMod {
            mod_info,
            installed_file: selected_file,
            install_path: file_path,
            enabled: true,
            install_date: chrono::Utc::now(),
            update_available: None,
        };
        
        // Save to installed mods
        self.installed_mods.insert(mod_id.to_string(), installed_mod.clone());
        self.save_installed_mods().await?;
        
        Ok(installed_mod)
    }
    
    /// Uninstall a mod
    pub async fn uninstall_mod(&mut self, mod_id: &str) -> Result<(), ModError> {
        let installed_mod = self.installed_mods.remove(mod_id)
            .ok_or_else(|| ModError::NotFound(format!("Mod {} not installed", mod_id)))?;
        
        // Remove the mod file
        if installed_mod.install_path.exists() {
            fs::remove_file(&installed_mod.install_path).await
                .context("Failed to remove mod file")?;
        }
        
        // Save updated state
        self.save_installed_mods().await?;
        
        Ok(())
    }
    
    /// Update a mod to the latest version
    pub async fn update_mod<F>(&mut self, mod_id: &str, progress_callback: F) -> Result<(), ModError>
    where
        F: Fn(u64, u64) + Send + Sync + Clone + 'static,
    {
        let installed_mod = self.installed_mods.get(mod_id)
            .ok_or_else(|| ModError::NotFound(format!("Mod {} not installed", mod_id)))?;
        
        // Find the appropriate API client
        let mut selected_client = None;
        for client in &self.api_clients {
            if let Ok(update) = client.check_updates(installed_mod).await {
                if update.is_some() {
                    selected_client = Some(client);
                    break;
                }
            }
        }
        
        let client = selected_client.ok_or_else(|| ModError::NotFound("No updates available".to_string()))?;
        let update = client.check_updates(installed_mod).await?
            .ok_or_else(|| ModError::NotFound("No updates available".to_string()))?;
        
        // Backup old file
        let backup_path = installed_mod.install_path.with_extension("bak");
        fs::rename(&installed_mod.install_path, &backup_path).await
            .context("Failed to backup old mod file")?;
        
        // Download new version
        let progress_box: Box<dyn Fn(u64, u64) + Send + Sync> = Box::new(progress_callback);
        match client.download_mod_file(&update, &installed_mod.install_path, progress_box).await {
            Ok(_) => {
                // Update was successful, remove backup
                let _ = fs::remove_file(&backup_path).await;
                
                // Update installed mod record
                let mut updated_mod = installed_mod.clone();
                updated_mod.installed_file = update;
                updated_mod.update_available = None;
                self.installed_mods.insert(mod_id.to_string(), updated_mod);
                self.save_installed_mods().await?;
                
                Ok(())
            }
            Err(e) => {
                // Restore backup on failure
                let _ = fs::rename(&backup_path, &installed_mod.install_path).await;
                Err(e)
            }
        }
    }
    
    /// Check for updates for all installed mods
    pub async fn check_all_updates(&mut self) -> Result<Vec<String>, ModError> {
        let mut mods_with_updates = Vec::new();
        
        for (mod_id, installed_mod) in &mut self.installed_mods {
            for client in &self.api_clients {
                if let Ok(Some(update)) = client.check_updates(installed_mod).await {
                    installed_mod.update_available = Some(update);
                    mods_with_updates.push(mod_id.clone());
                    break;
                }
            }
        }
        
        self.save_installed_mods().await?;
        Ok(mods_with_updates)
    }
    
    /// Enable or disable a mod
    pub async fn set_mod_enabled(&mut self, mod_id: &str, enabled: bool) -> Result<(), ModError> {
        let installed_mod = self.installed_mods.get_mut(mod_id)
            .ok_or_else(|| ModError::NotFound(format!("Mod {} not installed", mod_id)))?;
        
        if enabled && !installed_mod.enabled {
            // Enable: ensure file has .jar extension
            if installed_mod.install_path.extension().unwrap_or_default() != "jar" {
                let new_path = installed_mod.install_path.with_extension("jar");
                fs::rename(&installed_mod.install_path, &new_path).await
                    .context("Failed to enable mod")?;
                installed_mod.install_path = new_path;
            }
        } else if !enabled && installed_mod.enabled {
            // Disable: add .disabled extension
            let new_path = installed_mod.install_path.with_extension("jar.disabled");
            fs::rename(&installed_mod.install_path, &new_path).await
                .context("Failed to disable mod")?;
            installed_mod.install_path = new_path;
        }
        
        installed_mod.enabled = enabled;
        self.save_installed_mods().await?;
        Ok(())
    }
    
    /// Get all installed mods
    pub fn get_installed_mods(&self) -> &HashMap<String, InstalledMod> {
        &self.installed_mods
    }
    
    /// Search for mods across all APIs
    pub async fn search_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        let mut all_results = Vec::new();
        
        for client in &self.api_clients {
            match client.search_mods(query, game_version, mod_loader, limit, 0).await {
                Ok(mut results) => all_results.append(&mut results),
                Err(_) => continue, // Skip failed API calls
            }
        }
        
        // Remove duplicates based on name and description
        all_results.sort_by(|a, b| a.name.cmp(&b.name));
        all_results.dedup_by(|a, b| a.name == b.name && a.description == b.description);
        
        Ok(all_results)
    }
    
    /// Check dependencies for a mod file
    pub async fn check_dependencies(&self, mod_file: &ModFile) -> Result<Vec<ModDependency>, ModError> {
        let mut missing_deps = Vec::new();
        
        for dep in &mod_file.dependencies {
            if dep.dependency_type == DependencyType::Required {
                if !self.installed_mods.contains_key(&dep.mod_id) {
                    missing_deps.push(dep.clone());
                }
            }
        }
        
        if !missing_deps.is_empty() {
            return Err(ModError::DependencyConflict(
                format!("Missing required dependencies: {}", 
                    missing_deps.iter()
                        .map(|d| d.mod_id.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            ));
        }
        
        Ok(missing_deps)
    }
    
    /// Install missing dependencies for a mod
    pub async fn install_dependencies<F>(&mut self, mod_file: &ModFile, progress_callback: F) -> Result<Vec<InstalledMod>, ModError>
    where
        F: Fn(u64, u64) + Send + Sync + Clone + 'static,
    {
        let mut installed_deps = Vec::new();
        
        for dep in &mod_file.dependencies {
            if dep.dependency_type == DependencyType::Required && !self.installed_mods.contains_key(&dep.mod_id) {
                match self.install_mod(&dep.mod_id, dep.version_id.as_deref(), progress_callback.clone()).await {
                    Ok(installed) => installed_deps.push(installed),
                    Err(e) => return Err(e),
                }
            }
        }
        
        Ok(installed_deps)
    }
    
    /// Refresh the list of installed mods by scanning the filesystem
    pub async fn refresh_installed_mods(&mut self) -> Result<(), ModError> {
        // Clear current list
        self.installed_mods.clear();
        
        // Load from saved metadata if available
        if let Ok(metadata) = self.load_installed_mods().await {
            self.installed_mods = metadata;
        }
        
        // Scan filesystem for mod files
        if let Ok(mut entries) = fs::read_dir(&self.mods_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "jar" {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                        
                        // If we don't have metadata for this file, create a basic entry
                        if !self.installed_mods.values().any(|m| m.install_path == path) {
                            // This is a basic entry for mods without metadata
                            // In a real implementation, we might try to parse the mod's metadata
                            continue;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Save installed mods metadata
    async fn save_installed_mods(&self) -> Result<(), ModError> {
        let metadata_path = self.instance_path.join("mods_metadata.json");
        let json = serde_json::to_string_pretty(&self.installed_mods)?;
        fs::write(metadata_path, json).await
            .context("Failed to save mods metadata")?;
        Ok(())
    }
    
    /// Load installed mods metadata
    async fn load_installed_mods(&self) -> Result<HashMap<String, InstalledMod>, ModError> {
        let metadata_path = self.instance_path.join("mods_metadata.json");
        if !metadata_path.exists() {
            return Ok(HashMap::new());
        }
        
        let json = fs::read_to_string(metadata_path).await
            .context("Failed to read mods metadata")?;
        let metadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }
}