use crate::mods::types::*;
use tokio::fs;
use serde_json;
use reqwest;

/// Mod loader installer and manager
pub struct ModLoaderManager {
    instance_path: std::path::PathBuf,
}

impl ModLoaderManager {
    pub fn new(instance_path: std::path::PathBuf) -> Self {
        Self { instance_path }
    }
    
    /// Install a mod loader for the instance
    pub async fn install_loader(&self, loader: &ModLoader, mc_version: &str) -> Result<(), ModError> {
        match loader {
            ModLoader::Forge(version) => self.install_forge(version, mc_version).await,
            ModLoader::Fabric(version) => self.install_fabric(version, mc_version).await,
            ModLoader::Quilt(version) => self.install_quilt(version, mc_version).await,
            ModLoader::NeoForge(version) => self.install_neoforge(version, mc_version).await,
            _ => Err(ModError::LoaderNotSupported(format!("Loader {:?} not yet supported", loader))),
        }
    }
    
    /// Check if a mod loader is installed for the instance
    pub async fn is_loader_installed(&self, loader: &ModLoader) -> bool {
        match loader {
            ModLoader::Forge(_) => self.is_forge_installed().await,
            ModLoader::Fabric(_) => self.is_fabric_installed().await,
            ModLoader::Quilt(_) => self.is_quilt_installed().await,
            ModLoader::NeoForge(_) => self.is_neoforge_installed().await,
            _ => false,
        }
    }
    
    /// Get the installed mod loader for the instance
    pub async fn get_installed_loader(&self) -> Option<ModLoader> {
        // Check for various loader indicators
        if self.is_forge_installed().await {
            self.get_forge_version().await.map(ModLoader::Forge)
        } else if self.is_fabric_installed().await {
            self.get_fabric_version().await.map(ModLoader::Fabric)
        } else if self.is_quilt_installed().await {
            self.get_quilt_version().await.map(ModLoader::Quilt)
        } else if self.is_neoforge_installed().await {
            self.get_neoforge_version().await.map(ModLoader::NeoForge)
        } else {
            None
        }
    }
    
    /// Get available mod loader versions for a minecraft version
    pub async fn get_available_versions(&self, loader_name: &str, mc_version: &str) -> Result<Vec<String>, ModError> {
        match loader_name.to_lowercase().as_str() {
            "forge" => self.get_forge_versions(mc_version).await,
            "fabric" => self.get_fabric_versions(mc_version).await,
            "quilt" => self.get_quilt_versions(mc_version).await,
            "neoforge" => self.get_neoforge_versions(mc_version).await,
            _ => Ok(Vec::new()),
        }
    }
    
    // Private implementation methods
    
    async fn install_forge(&self, version: &str, mc_version: &str) -> Result<(), ModError> {
        // For now, create a simple marker file to indicate Forge installation
        // This is a placeholder until full Forge installation is implemented
        println!("Mock installing Forge {} for MC {}", version, mc_version);
        
        let forge_marker = self.instance_path.join("mods").join(".forge_installed");
        if let Ok(parent) = forge_marker.parent().ok_or_else(|| ModError::InvalidFile("Invalid path".to_string())) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&forge_marker, format!("forge-{}", version)).await?;
        
        Ok(())
    }
    
    async fn install_fabric(&self, version: &str, mc_version: &str) -> Result<(), ModError> {
        // For now, create a simple marker file to indicate Fabric installation
        // This is a placeholder until full Fabric installation is implemented
        println!("Mock installing Fabric {} for MC {}", version, mc_version);
        
        let fabric_marker = self.instance_path.join("mods").join(".fabric_installed");
        if let Ok(parent) = fabric_marker.parent().ok_or_else(|| ModError::InvalidFile("Invalid path".to_string())) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&fabric_marker, format!("fabric-{}", version)).await?;
        
        Ok(())
    }
    
    async fn install_quilt(&self, version: &str, mc_version: &str) -> Result<(), ModError> {
        // For now, create a simple marker file to indicate Quilt installation
        println!("Mock installing Quilt {} for MC {}", version, mc_version);
        
        let quilt_marker = self.instance_path.join("mods").join(".quilt_installed");
        if let Ok(parent) = quilt_marker.parent().ok_or_else(|| ModError::InvalidFile("Invalid path".to_string())) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&quilt_marker, format!("quilt-{}", version)).await?;
        
        Ok(())
    }
    
    async fn install_neoforge(&self, version: &str, mc_version: &str) -> Result<(), ModError> {
        // For now, create a simple marker file to indicate NeoForge installation
        println!("Mock installing NeoForge {} for MC {}", version, mc_version);
        
        let neoforge_marker = self.instance_path.join("mods").join(".neoforge_installed");
        if let Ok(parent) = neoforge_marker.parent().ok_or_else(|| ModError::InvalidFile("Invalid path".to_string())) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&neoforge_marker, format!("neoforge-{}", version)).await?;
        
        Ok(())
    }
    
    async fn is_forge_installed(&self) -> bool {
        // Check for Forge indicators (forge profile, forge libraries, etc.)
        let forge_marker = self.instance_path.join("mods").join(".forge_installed");
        if forge_marker.exists() {
            return true;
        }
        
        let libraries_path = self.instance_path.join("libraries");
        if let Ok(entries) = fs::read_dir(&libraries_path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.file_name().to_string_lossy().contains("forge") {
                    return true;
                }
            }
        }
        false
    }
    
    async fn is_fabric_installed(&self) -> bool {
        // Check for Fabric indicators
        let fabric_marker = self.instance_path.join("mods").join(".fabric_installed");
        if fabric_marker.exists() {
            return true;
        }
        
        let mods_path = self.instance_path.join("mods");
        if let Ok(entries) = fs::read_dir(&mods_path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("fabric-api") || name.contains("fabric-loader") {
                    return true;
                }
            }
        }
        false
    }
    
    async fn is_quilt_installed(&self) -> bool {
        // Check for Quilt indicators
        let quilt_marker = self.instance_path.join("mods").join(".quilt_installed");
        if quilt_marker.exists() {
            return true;
        }
        
        let mods_path = self.instance_path.join("mods");
        if let Ok(entries) = fs::read_dir(&mods_path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("quilt") {
                    return true;
                }
            }
        }
        false
    }
    
    async fn is_neoforge_installed(&self) -> bool {
        // Check for NeoForge indicators
        let neoforge_marker = self.instance_path.join("mods").join(".neoforge_installed");
        if neoforge_marker.exists() {
            return true;
        }
        
        let libraries_path = self.instance_path.join("libraries");
        if let Ok(entries) = fs::read_dir(&libraries_path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.file_name().to_string_lossy().contains("neoforge") {
                    return true;
                }
            }
        }
        false
    }
    
    async fn get_forge_version(&self) -> Option<String> {
        // Try to read version from marker file first
        let forge_marker = self.instance_path.join("mods").join(".forge_installed");
        if let Ok(content) = tokio::fs::read_to_string(&forge_marker).await {
            if let Some(version) = content.strip_prefix("forge-") {
                return Some(version.trim().to_string());
            }
        }
        
        // Try to parse from forge installation files
        let libraries_path = self.instance_path.join("libraries");
        if let Ok(mut entries) = fs::read_dir(&libraries_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("forge") && name.contains("-") {
                    // Extract version from filename like "forge-1.20.1-47.2.0.jar"
                    let parts: Vec<&str> = name.split('-').collect();
                    if parts.len() >= 3 {
                        return Some(parts[2].replace(".jar", ""));
                    }
                }
            }
        }
        
        None
    }
    
    async fn get_fabric_version(&self) -> Option<String> {
        // Try to read version from marker file first
        let fabric_marker = self.instance_path.join("mods").join(".fabric_installed");
        if let Ok(content) = tokio::fs::read_to_string(&fabric_marker).await {
            if let Some(version) = content.strip_prefix("fabric-") {
                return Some(version.trim().to_string());
            }
        }
        
        // Try to parse from fabric loader files in mods directory
        let mods_path = self.instance_path.join("mods");
        if let Ok(mut entries) = fs::read_dir(&mods_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("fabric-loader") && name.contains("-") {
                    // Extract version from filename like "fabric-loader-0.14.24.jar"
                    let parts: Vec<&str> = name.split('-').collect();
                    if parts.len() >= 3 {
                        return Some(parts[2].replace(".jar", ""));
                    }
                }
            }
        }
        
        None
    }
    
    async fn get_quilt_version(&self) -> Option<String> {
        // Try to read version from marker file first
        let quilt_marker = self.instance_path.join("mods").join(".quilt_installed");
        if let Ok(content) = tokio::fs::read_to_string(&quilt_marker).await {
            if let Some(version) = content.strip_prefix("quilt-") {
                return Some(version.trim().to_string());
            }
        }
        
        // Try to parse from quilt files in mods directory
        let mods_path = self.instance_path.join("mods");
        if let Ok(mut entries) = fs::read_dir(&mods_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("quilt-loader") && name.contains("-") {
                    // Extract version from filename like "quilt-loader-0.21.1.jar"
                    let parts: Vec<&str> = name.split('-').collect();
                    if parts.len() >= 3 {
                        return Some(parts[2].replace(".jar", ""));
                    }
                }
            }
        }
        
        None
    }
    
    async fn get_neoforge_version(&self) -> Option<String> {
        // Try to read version from marker file first
        let neoforge_marker = self.instance_path.join("mods").join(".neoforge_installed");
        if let Ok(content) = tokio::fs::read_to_string(&neoforge_marker).await {
            if let Some(version) = content.strip_prefix("neoforge-") {
                return Some(version.trim().to_string());
            }
        }
        
        // Try to parse from neoforge installation files
        let libraries_path = self.instance_path.join("libraries");
        if let Ok(mut entries) = fs::read_dir(&libraries_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("neoforge") && name.contains("-") {
                    // Extract version from filename like "neoforge-1.20.1-20.4.109.jar"
                    let parts: Vec<&str> = name.split('-').collect();
                    if parts.len() >= 3 {
                        return Some(parts[2].replace(".jar", ""));
                    }
                }
            }
        }
        
        None
    }
    
    async fn get_forge_versions(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        // Try to fetch from Forge API
        match self.fetch_forge_versions_from_api(mc_version).await {
            Ok(versions) => Ok(versions),
            Err(_) => {
                // Fallback to common versions if API fails
                Ok(vec![
                    "47.2.20".to_string(), 
                    "47.2.0".to_string(), 
                    "47.1.0".to_string(),
                    "46.0.14".to_string(),
                ])
            }
        }
    }
    
    async fn fetch_forge_versions_from_api(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        let client = reqwest::Client::new();
        let url = format!("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_{}.json", mc_version);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(promos) = data.get("promos") {
                let mut versions = Vec::new();
                
                // Get recommended and latest versions
                if let Some(recommended) = promos.get(&format!("{}-recommended", mc_version)) {
                    if let Some(version_str) = recommended.as_str() {
                        versions.push(version_str.to_string());
                    }
                }
                if let Some(latest) = promos.get(&format!("{}-latest", mc_version)) {
                    if let Some(version_str) = latest.as_str() {
                        if !versions.contains(&version_str.to_string()) {
                            versions.push(version_str.to_string());
                        }
                    }
                }
                
                if !versions.is_empty() {
                    return Ok(versions);
                }
            }
        }
        
        Err(ModError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No versions found"
        )))
    }
    
    async fn get_fabric_versions(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        // Try to fetch from Fabric API
        match self.fetch_fabric_versions_from_api(mc_version).await {
            Ok(versions) => Ok(versions),
            Err(_) => {
                // Fallback to common versions if API fails
                Ok(vec![
                    "0.15.3".to_string(),
                    "0.14.24".to_string(), 
                    "0.14.23".to_string(),
                    "0.14.22".to_string(),
                ])
            }
        }
    }
    
    async fn fetch_fabric_versions_from_api(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        let client = reqwest::Client::new();
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(versions_array) = data.as_array() {
                let versions: Vec<String> = versions_array.iter()
                    .filter_map(|v| v.get("loader").and_then(|l| l.get("version")).and_then(|ver| ver.as_str()))
                    .take(10) // Limit to first 10 versions
                    .map(|s| s.to_string())
                    .collect();
                
                if !versions.is_empty() {
                    return Ok(versions);
                }
            }
        }
        
        Err(ModError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No versions found"
        )))
    }
    
    async fn get_quilt_versions(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        // Try to fetch from Quilt API
        match self.fetch_quilt_versions_from_api(mc_version).await {
            Ok(versions) => Ok(versions),
            Err(_) => {
                // Fallback to common versions if API fails
                Ok(vec![
                    "0.21.1".to_string(), 
                    "0.21.0".to_string(),
                    "0.20.2".to_string(),
                    "0.20.1".to_string(),
                ])
            }
        }
    }
    
    async fn fetch_quilt_versions_from_api(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        let client = reqwest::Client::new();
        let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(versions_array) = data.as_array() {
                let versions: Vec<String> = versions_array.iter()
                    .filter_map(|v| v.get("loader").and_then(|l| l.get("version")).and_then(|ver| ver.as_str()))
                    .take(10) // Limit to first 10 versions
                    .map(|s| s.to_string())
                    .collect();
                
                if !versions.is_empty() {
                    return Ok(versions);
                }
            }
        }
        
        Err(ModError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No versions found"
        )))
    }
    
    async fn get_neoforge_versions(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        // Try to fetch from NeoForge API
        match self.fetch_neoforge_versions_from_api(mc_version).await {
            Ok(versions) => Ok(versions),
            Err(_) => {
                // Fallback to common versions if API fails
                Ok(vec![
                    "20.4.195".to_string(),
                    "20.4.109".to_string(), 
                    "20.4.108".to_string(),
                    "20.2.88".to_string(),
                ])
            }
        }
    }
    
    async fn fetch_neoforge_versions_from_api(&self, mc_version: &str) -> Result<Vec<String>, ModError> {
        let client = reqwest::Client::new();
        let url = format!("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge");
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(versions_array) = data.get("versions").and_then(|v| v.as_array()) {
                let versions: Vec<String> = versions_array.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|v| v.starts_with(&format!("{}.", mc_version.replace("1.", ""))))
                    .take(10) // Limit to first 10 versions
                    .map(|s| s.to_string())
                    .collect();
                
                if !versions.is_empty() {
                    return Ok(versions);
                }
            }
        }
        
        Err(ModError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No versions found"
        )))
    }
}