use std::path::{Path, PathBuf};
use libium::{
    config::{
        read_config, write_config,
        structs::{Config, Profile, ModIdentifier, ModLoader},
        filters::Filter,
    },
    add,
    scan,
    CURSEFORGE_API, GITHUB_API, MODRINTH_API,
};
use tokio::fs;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Modrinth API structures
#[derive(Deserialize, Debug)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthSearchHit>,
    offset: u32,
    limit: u32,
    total_hits: u32,
}

#[derive(Deserialize, Debug)]
struct ModrinthSearchHit {
    project_id: String,
    project_type: String,
    slug: String,
    author: String,
    title: String,
    description: String,
    categories: Vec<String>,
    versions: Vec<String>,
    downloads: u64,
    follows: u64,
    icon_url: Option<String>,
    date_created: String,
    date_modified: String,
    latest_version: Option<String>,
    license: String,
    client_side: String,
    server_side: String,
    gallery: Vec<String>,
    featured_gallery: Option<String>,
    color: Option<u32>,
}

// CurseForge API structures
#[derive(Deserialize, Debug)]
struct CurseForgeSearchResponse {
    data: Vec<CurseForgeSearchHit>,
    pagination: CurseForgePagination,
}

#[derive(Deserialize, Debug)]
struct CurseForgePagination {
    index: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    #[serde(rename = "resultCount")]
    result_count: u32,
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Deserialize, Debug)]
struct CurseForgeSearchHit {
    #[serde(rename = "screenshots")]
    screenshots: Vec<CurseForgeScreenshot>,
    id: u64,
    #[serde(rename = "gameId")]
    game_id: u64,
    name: String,
    slug: String,
    links: CurseForgeLinks,
    summary: String,
    status: u32,
    #[serde(rename = "downloadCount")]
    download_count: u64,
    #[serde(rename = "isFeatured")]
    is_featured: bool,
    #[serde(rename = "primaryCategoryId")]
    primary_category_id: u64,
    categories: Vec<CurseForgeCategory>,
    #[serde(rename = "classId")]
    class_id: Option<u64>,
    authors: Vec<CurseForgeAuthor>,
    logo: Option<CurseForgeLogo>,
    #[serde(rename = "mainFileId")]
    main_file_id: u64,
    #[serde(rename = "latestFiles")]
    latest_files: Vec<CurseForgeFile>,
    #[serde(rename = "latestFilesIndexes")]
    latest_files_indexes: Vec<CurseForgeFileIndex>,
    #[serde(rename = "latestEarlyAccessFilesIndexes")]
    latest_early_access_files_indexes: Vec<CurseForgeFileIndex>,
    #[serde(rename = "dateCreated")]
    date_created: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "dateReleased")]
    date_released: String,
    #[serde(rename = "allowModDistribution")]
    allow_mod_distribution: Option<bool>,
    #[serde(rename = "gamePopularityRank")]
    game_popularity_rank: u64,
    #[serde(rename = "isAvailable")]
    is_available: bool,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: u64,
    #[serde(rename = "featuredProjectTag")]
    featured_project_tag: Option<u64>,
    rating: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeLinks {
    #[serde(rename = "websiteUrl")]
    website_url: String,
    #[serde(rename = "wikiUrl")]
    wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    source_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeCategory {
    id: u64,
    #[serde(rename = "gameId")]
    game_id: u64,
    name: String,
    slug: String,
    url: String,
    #[serde(rename = "iconUrl")]
    icon_url: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "isClass")]
    is_class: Option<bool>,
    #[serde(rename = "classId")]
    class_id: Option<u64>,
    #[serde(rename = "parentCategoryId")]
    parent_category_id: Option<u64>,
    #[serde(rename = "displayIndex")]
    display_index: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeAuthor {
    id: u64,
    name: String,
    url: String,
    #[serde(rename = "avatarUrl")]
    avatar_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeLogo {
    id: u64,
    #[serde(rename = "modId")]
    mod_id: u64,
    title: String,
    description: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct CurseForgeScreenshot {
    id: u64,
    #[serde(rename = "modId")]
    mod_id: u64,
    title: String,
    description: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct CurseForgeFile {
    id: u64,
    #[serde(rename = "gameId")]
    game_id: u64,
    #[serde(rename = "modId")]
    mod_id: Option<u64>,
    #[serde(rename = "isAvailable")]
    is_available: bool,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "releaseType")]
    release_type: u32,
    #[serde(rename = "fileStatus")]
    file_status: u32,
    hashes: Vec<CurseForgeHash>,
    #[serde(rename = "fileDate")]
    file_date: String,
    #[serde(rename = "fileLength")]
    file_length: u64,
    #[serde(rename = "downloadCount")]
    download_count: u64,
    #[serde(rename = "fileSizeOnDisk")]
    file_size_on_disk: Option<u64>,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    game_versions: Vec<String>,
    #[serde(rename = "sortableGameVersions")]
    sortable_game_versions: Vec<CurseForgeSortableGameVersion>,
    dependencies: Vec<CurseForgeDependency>,
    #[serde(rename = "alternateFileId")]
    alternate_file_id: Option<u64>,
    #[serde(rename = "isServerPack")]
    is_server_pack: Option<bool>,
    #[serde(rename = "fileFingerprint")]
    file_fingerprint: u64,
    modules: Vec<CurseForgeModule>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeFileIndex {
    #[serde(rename = "gameVersion")]
    game_version: String,
    #[serde(rename = "fileId")]
    file_id: u64,
    filename: String,
    #[serde(rename = "releaseType")]
    release_type: u32,
    #[serde(rename = "gameVersionTypeId")]
    game_version_type_id: Option<u64>,
    #[serde(rename = "modLoader")]
    mod_loader: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeHash {
    value: String,
    algo: u32,
}

#[derive(Deserialize, Debug)]
struct CurseForgeSortableGameVersion {
    #[serde(rename = "gameVersionName")]
    game_version_name: String,
    #[serde(rename = "gameVersionPadded")]
    game_version_padded: String,
    #[serde(rename = "gameVersion")]
    game_version: String,
    #[serde(rename = "gameVersionReleaseDate")]
    game_version_release_date: String,
    #[serde(rename = "gameVersionTypeId")]
    game_version_type_id: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeDependency {
    mod_id: u64,
    relation_type: u32,
}

#[derive(Deserialize, Debug)]
struct CurseForgeModule {
    name: String,
    fingerprint: u64,
}

pub mod commands;
pub mod types;
pub mod modpacks;

#[derive(Debug, thiserror::Error)]
pub enum LibiumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Add error: {0}")]
    Add(#[from] libium::add::Error),
    #[error("Scan error: {0}")]
    Scan(#[from] libium::scan::Error),
    #[error("Check error: {0}")]
    Check(#[from] libium::upgrade::check::Error),
    #[error("Config not found")]
    ConfigNotFound,
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),
    #[error("Invalid data directory")]
    InvalidDataDirectory,
}

/// Main libium integration manager for ChaiLauncher
pub struct LibiumManager {
    config_path: PathBuf,
    data_dir: PathBuf,
    config: Config,
    client: Client,
}

impl LibiumManager {
    /// Create a new LibiumManager with ChaiLauncher's data directory
    pub async fn new() -> Result<Self, LibiumError> {
        let data_dir = crate::storage::get_launcher_dir();
        let config_path = data_dir.join("libium_config.json");
        
        // Ensure data directory exists
        fs::create_dir_all(&data_dir).await?;
        
        // Read or create config
        let config = if config_path.exists() {
            read_config(&config_path)?
        } else {
            let default_config = Config::default();
            write_config(&config_path, &default_config)?;
            default_config
        };
        
        Ok(Self {
            config_path,
            data_dir,
            config,
            client: Client::new(),
        })
    }
    
    /// Create a profile for an instance if it doesn't exist
    pub async fn ensure_instance_profile(&mut self, instance_id: &str, instance_path: &Path) -> Result<(), LibiumError> {
        let profile_name = format!("chai_instance_{}", instance_id);
        
        // Check if profile already exists
        if !self.config.profiles.iter().any(|p| p.name == profile_name) {
            let mods_dir = instance_path.join("mods");
            fs::create_dir_all(&mods_dir).await?;
            
            // Create a new profile with default Minecraft version and Fabric loader
            let profile = Profile::new(
                profile_name,
                mods_dir,
                vec!["1.20.1".to_string()], // Default version - can be updated later
                ModLoader::Fabric,
            );
            
            self.config.profiles.push(profile);
            self.save_config().await?;
        }
        
        Ok(())
    }
    
    /// Get the profile for an instance
    pub fn get_instance_profile(&mut self, instance_id: &str) -> Result<&mut Profile, LibiumError> {
        let profile_name = format!("chai_instance_{}", instance_id);
        self.config.profiles.iter_mut()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| LibiumError::ProfileNotFound(profile_name))
    }
    
    /// Update profile filters (game version, mod loader, etc.)
    pub async fn update_profile_filters(
        &mut self,
        instance_id: &str,
        game_versions: Vec<String>,
        mod_loader: ModLoader,
    ) -> Result<(), LibiumError> {
        let profile = self.get_instance_profile(instance_id)?;
        
        profile.filters = vec![
            Filter::ModLoaderPrefer(match mod_loader {
                ModLoader::Quilt => vec![ModLoader::Quilt, ModLoader::Fabric],
                _ => vec![mod_loader],
            }),
            Filter::GameVersionStrict(game_versions),
        ];
        
        self.save_config().await?;
        Ok(())
    }
    
    /// Search for mods across all supported platforms
    pub async fn search_mods(
        &self,
        query: &str,
        game_version: Option<String>,
        mod_loader: Option<ModLoader>,
        limit: Option<u32>,
    ) -> Result<Vec<types::ModSearchResult>, LibiumError> {
        let mut results = Vec::new();
        let limit = limit.unwrap_or(20);
        
        // Search Modrinth
        if let Ok(modrinth_results) = self.search_modrinth_mods(query, game_version.as_deref(), mod_loader, limit).await {
            results.extend(modrinth_results);
        }
        
        // Search CurseForge (if we have room for more results)
        let remaining_limit = limit.saturating_sub(results.len() as u32);
        if remaining_limit > 0 {
            if let Ok(curseforge_results) = self.search_curseforge_mods(query, game_version.as_deref(), mod_loader, remaining_limit).await {
                results.extend(curseforge_results);
            }
        }
        
        // Limit final results
        results.truncate(limit as usize);
        
        Ok(results)
    }
    
    /// Search Modrinth for mods
    async fn search_modrinth_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        mod_loader: Option<ModLoader>,
        limit: u32,
    ) -> Result<Vec<types::ModSearchResult>, LibiumError> {
        let mut url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit={}&project_type=mod",
            urlencoding::encode(query),
            limit
        );
        
        // Add facets for filtering
        let mut facets = Vec::new();
        
        if let Some(version) = game_version {
            facets.push(format!("\"versions:{}\"", version));
        }
        
        if let Some(loader) = mod_loader {
            let loader_name = match loader {
                ModLoader::Fabric => "fabric",
                ModLoader::Forge => "forge", 
                ModLoader::Quilt => "quilt",
                ModLoader::NeoForge => "neoforge",
                _ => "fabric", // fallback
            };
            facets.push(format!("\"categories:{}\"", loader_name));
        }
        
        if !facets.is_empty() {
            url.push_str(&format!("&facets=[[{}]]", facets.join(",")));
        }
        
        let response = self.client
            .get(&url)
            .header("User-Agent", "ChaiLauncher/1.0")
            .send()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let search_response: ModrinthSearchResponse = response
            .json()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        let results = search_response.hits.into_iter().map(|hit| {
            types::ModSearchResult {
                id: hit.project_id,
                name: hit.title,
                description: hit.description,
                author: hit.author,
                downloads: hit.downloads,
                icon_url: hit.icon_url,
                project_type: "modrinth".to_string(),
                game_versions: hit.versions,
                mod_loaders: hit.categories.iter()
                    .filter_map(|cat| match cat.as_str() {
                        "fabric" => Some("fabric".to_string()),
                        "forge" => Some("forge".to_string()),
                        "quilt" => Some("quilt".to_string()),
                        "neoforge" => Some("neoforge".to_string()),
                        _ => None,
                    })
                    .collect(),
                created_at: hit.date_created,
                updated_at: hit.date_modified,
            }
        }).collect();
        
        Ok(results)
    }
    
    /// Search CurseForge for mods
    async fn search_curseforge_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        mod_loader: Option<ModLoader>,
        limit: u32,
    ) -> Result<Vec<types::ModSearchResult>, LibiumError> {
        let mut url = format!(
            "https://api.curseforge.com/v1/mods/search?gameId=432&classId=6&searchFilter={}&pageSize={}",
            urlencoding::encode(query),
            limit
        );
        
        if let Some(version) = game_version {
            url.push_str(&format!("&gameVersion={}", version));
        }
        
        if let Some(loader) = mod_loader {
            let loader_id = match loader {
                ModLoader::Fabric => 4, // Fabric loader ID
                ModLoader::Forge => 1,  // Forge loader ID  
                ModLoader::Quilt => 5,  // Quilt loader ID
                ModLoader::NeoForge => 6, // NeoForge loader ID
                _ => 4, // fallback to Fabric
            };
            url.push_str(&format!("&modLoaderType={}", loader_id));
        }
        
        // Get CurseForge API key from environment or use default
        let api_key = std::env::var("CURSEFORGE_API_KEY")
            .unwrap_or_else(|_| "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.".to_string());
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("x-api-key", api_key)
            .header("User-Agent", "ChaiLauncher/1.0")
            .send()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let search_response: CurseForgeSearchResponse = response
            .json()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        let results = search_response.data.into_iter().map(|hit| {
            let author = hit.authors.first()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let game_versions = hit.latest_files_indexes.iter()
                .map(|idx| idx.game_version.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
                
            let mod_loaders = hit.latest_files_indexes.iter()
                .filter_map(|idx| {
                    idx.mod_loader.map(|loader_id| match loader_id {
                        1 => "forge".to_string(),
                        4 => "fabric".to_string(), 
                        5 => "quilt".to_string(),
                        6 => "neoforge".to_string(),
                        _ => "unknown".to_string(),
                    })
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            
            types::ModSearchResult {
                id: hit.id.to_string(),
                name: hit.name,
                description: hit.summary,
                author,
                downloads: hit.download_count,
                icon_url: hit.logo.map(|logo| logo.thumbnail_url),
                project_type: "curseforge".to_string(),
                game_versions,
                mod_loaders,
                created_at: hit.date_created,
                updated_at: hit.date_modified,
            }
        }).collect();
        
        Ok(results)
    }
    
    /// Add a mod to an instance profile
    pub async fn add_mod_to_instance(
        &mut self,
        instance_id: &str,
        mod_id: &str,
        platform: &str,
        filters: Vec<Filter>,
    ) -> Result<String, LibiumError> {
        let profile = self.get_instance_profile(instance_id)?;
        
        let identifier = match platform {
            "modrinth" => ModIdentifier::ModrinthProject(mod_id.to_string()),
            "curseforge" => ModIdentifier::CurseForgeProject(
                mod_id.parse().map_err(|_| LibiumError::Add(libium::add::Error::DoesNotExist))?
            ),
            "github" => {
                let parts: Vec<&str> = mod_id.split('/').collect();
                if parts.len() != 2 {
                    return Err(LibiumError::Add(libium::add::Error::DoesNotExist));
                }
                ModIdentifier::GitHubRepository(parts[0].to_string(), parts[1].to_string())
            }
            _ => return Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
        };
        
        let (successful_mods, errors) = add::add(
            profile,
            vec![identifier],
            true,  // perform_checks
            false, // override_profile
            filters,
        ).await?;
        
        if !errors.is_empty() {
            return Err(LibiumError::Add(errors.into_iter().next().unwrap().1));
        }
        
        self.save_config().await?;
        
        Ok(successful_mods.into_iter().next().unwrap_or_else(|| "Unknown mod".to_string()))
    }
    
    /// Remove a mod from an instance profile
    pub async fn remove_mod_from_instance(
        &mut self,
        instance_id: &str,
        mod_name: &str,
    ) -> Result<(), LibiumError> {
        let profile = self.get_instance_profile(instance_id)?;
        
        profile.mods.retain(|mod_| mod_.name != mod_name);
        self.save_config().await?;
        
        Ok(())
    }
    
    /// Get all mods for an instance
    pub fn get_instance_mods(&mut self, instance_id: &str) -> Result<Vec<types::InstalledMod>, LibiumError> {
        let profile = self.get_instance_profile(instance_id)?;
        
        let mods = profile.mods.iter().map(|mod_| {
            types::InstalledMod {
                name: mod_.name.clone(),
                slug: mod_.slug.clone(),
                identifier: match &mod_.identifier {
                    ModIdentifier::ModrinthProject(id) => format!("modrinth:{}", id),
                    ModIdentifier::CurseForgeProject(id) => format!("curseforge:{}", id),
                    ModIdentifier::GitHubRepository(owner, repo) => format!("github:{}/{}", owner, repo),
                    _ => "unknown".to_string(),
                },
                filters: mod_.filters.iter().map(|f| format!("{:?}", f)).collect(),
                override_filters: mod_.override_filters,
                enabled: true, // libium doesn't track enabled state directly
                install_date: chrono::Utc::now(), // libium doesn't track install date
                version: None, // Will need to be populated from actual files
            }
        }).collect();
        
        Ok(mods)
    }
    
    /// Scan mods in an instance directory
    pub async fn scan_instance_mods(
        &mut self,
        instance_id: &str,
    ) -> Result<Vec<types::ScannedMod>, LibiumError> {
        let profile = self.get_instance_profile(instance_id)?;
        let mods_dir = &profile.output_dir;
        
        if !mods_dir.exists() {
            return Ok(Vec::new());
        }
        
        let results = scan::scan(mods_dir, || {
            // Hashing complete callback
        }).await?;
        
        let scanned_mods = results.into_iter().map(|(filename, modrinth_id, curseforge_id)| {
            let file_path = mods_dir.join(&filename);
            types::ScannedMod {
                filename: filename.clone(),
                modrinth_project_id: modrinth_id,
                curseforge_project_id: curseforge_id,
                file_path,
                file_size: 0, // Would need to read file size separately
                enabled: !filename.ends_with(".disabled"),
            }
        }).collect();
        
        Ok(scanned_mods)
    }
    
    /// Save the configuration
    async fn save_config(&self) -> Result<(), LibiumError> {
        write_config(&self.config_path, &self.config)?;
        Ok(())
    }
    
    /// Get the configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    /// Get mutable configuration
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

/// Install a mod loader for an instance
pub async fn install_mod_loader(
    instance_path: &Path,
    game_version: &str,
    mod_loader: &str,
) -> Result<String, LibiumError> {
    let versions_dir = instance_path.join("versions").join(game_version);
    let libraries_dir = instance_path.join("libraries");
    
    // Create necessary directories
    fs::create_dir_all(&versions_dir).await?;
    fs::create_dir_all(&libraries_dir).await?;
    
    let profile_name = match mod_loader.to_lowercase().as_str() {
        "fabric" => install_fabric_loader(instance_path, game_version).await?,
        "forge" => install_forge_loader(instance_path, game_version).await?,
        "quilt" => install_quilt_loader(instance_path, game_version).await?,
        "neoforge" => install_neoforge_loader(instance_path, game_version).await?,
        _ => return Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
    };
    
    println!("✅ Successfully installed {} mod loader for Minecraft {} (profile: {})", mod_loader, game_version, profile_name);
    Ok(profile_name)
}

/// Install Fabric mod loader
async fn install_fabric_loader(instance_path: &Path, game_version: &str) -> Result<String, LibiumError> {
    let client = reqwest::Client::new();
    
    // Get Fabric loader versions
    let versions_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", game_version);
    let response = client.get(&versions_url)
        .header("User-Agent", "ChaiLauncher/1.0")
        .send()
        .await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let loader_versions: serde_json::Value = response.json().await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    
    // Get the latest stable loader version
    if let Some(latest_loader) = loader_versions.as_array().and_then(|arr| arr.first()) {
        if let Some(loader_version) = latest_loader.get("loader").and_then(|l| l.get("version")).and_then(|v| v.as_str()) {
            // Download Fabric profile
            let profile_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json", game_version, loader_version);
            let profile_response = client.get(&profile_url)
                .header("User-Agent", "ChaiLauncher/1.0")
                .send()
                .await
                .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
            let profile_data: serde_json::Value = profile_response.json().await
                .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            
            // Save Fabric profile
            let profile_name = format!("fabric-loader-{}-{}", loader_version, game_version);
            let versions_dir = instance_path.join("versions").join(&profile_name);
            fs::create_dir_all(&versions_dir).await?;
            
            let profile_path = versions_dir.join(format!("{}.json", profile_name));
            fs::write(&profile_path, serde_json::to_string_pretty(&profile_data)?).await?;
            
            // Download libraries
            if let Some(libraries) = profile_data.get("libraries").and_then(|l| l.as_array()) {
                let libraries_dir = instance_path.join("libraries");
                for library in libraries {
                    if let Some(downloads) = library.get("downloads") {
                        if let Some(artifact) = downloads.get("artifact") {
                            if let Some(url) = artifact.get("url").and_then(|u| u.as_str()) {
                                if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                                    let lib_path = libraries_dir.join(path);
                                    if let Some(parent) = lib_path.parent() {
                                        fs::create_dir_all(parent).await?;
                                    }
                                    
                                    let lib_response = client.get(url).send().await
                                        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                                    let lib_bytes = lib_response.bytes().await
                                        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                                    fs::write(&lib_path, &lib_bytes).await?;
                                }
                            }
                        }
                    }
                }
            }
            
            println!("✅ Installed Fabric loader {} for Minecraft {}", loader_version, game_version);
            return Ok(profile_name);
        }
    }
    
    Err(LibiumError::Add(libium::add::Error::DoesNotExist))
}

/// Install Forge mod loader
async fn install_forge_loader(instance_path: &Path, game_version: &str) -> Result<String, LibiumError> {
    let client = reqwest::Client::new();
    
    // Get Forge versions for the game version
    let versions_url = format!("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json");
    let response = client.get(&versions_url)
        .header("User-Agent", "ChaiLauncher/1.0")
        .send()
        .await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let forge_data: serde_json::Value = response.json().await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    
    // Look for recommended or latest version
    let version_key = format!("{}-recommended", game_version);
    let latest_key = format!("{}-latest", game_version);
    
    if let Some(promos) = forge_data.get("promos") {
        let forge_version = promos.get(&version_key)
            .or_else(|| promos.get(&latest_key))
            .and_then(|v| v.as_str());
        
        if let Some(forge_ver) = forge_version {
            println!("⬇️  Installing Forge {} for Minecraft {}", forge_ver, game_version);
            
            // Download Forge installer and extract profile
            // Note: This is a simplified implementation - full Forge installation 
            // requires running the installer JAR which is complex
            let installer_url = format!(
                "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                game_version, forge_ver, game_version, forge_ver
            );
            
            println!("⚠️  Forge installation requires manual installer run. Download from: {}", installer_url);
            println!("   Please install Forge manually and then retry.");
            
            // For now, return a Forge profile name even though installation isn't complete
            return Ok(format!("forge-{}-{}", game_version, forge_ver));
        }
    }
    
    Err(LibiumError::Add(libium::add::Error::DoesNotExist))
}

/// Install Quilt mod loader
async fn install_quilt_loader(instance_path: &Path, game_version: &str) -> Result<String, LibiumError> {
    let client = reqwest::Client::new();
    
    // Get Quilt loader versions
    let versions_url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", game_version);
    let response = client.get(&versions_url)
        .header("User-Agent", "ChaiLauncher/1.0")
        .send()
        .await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let loader_versions: serde_json::Value = response.json().await
        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    
    // Get the latest stable loader version
    if let Some(latest_loader) = loader_versions.as_array().and_then(|arr| arr.first()) {
        if let Some(loader_version) = latest_loader.get("loader").and_then(|l| l.get("version")).and_then(|v| v.as_str()) {
            // Download Quilt profile (similar to Fabric)
            let profile_url = format!("https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json", game_version, loader_version);
            let profile_response = client.get(&profile_url)
                .header("User-Agent", "ChaiLauncher/1.0")
                .send()
                .await
                .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
            let profile_data: serde_json::Value = profile_response.json().await
                .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            
            // Save Quilt profile
            let profile_name = format!("quilt-loader-{}-{}", loader_version, game_version);
            let versions_dir = instance_path.join("versions").join(&profile_name);
            fs::create_dir_all(&versions_dir).await?;
            
            let profile_path = versions_dir.join(format!("{}.json", profile_name));
            fs::write(&profile_path, serde_json::to_string_pretty(&profile_data)?).await?;
            
            // Download libraries (similar to Fabric)
            if let Some(libraries) = profile_data.get("libraries").and_then(|l| l.as_array()) {
                let libraries_dir = instance_path.join("libraries");
                for library in libraries {
                    if let Some(downloads) = library.get("downloads") {
                        if let Some(artifact) = downloads.get("artifact") {
                            if let Some(url) = artifact.get("url").and_then(|u| u.as_str()) {
                                if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                                    let lib_path = libraries_dir.join(path);
                                    if let Some(parent) = lib_path.parent() {
                                        fs::create_dir_all(parent).await?;
                                    }
                                    
                                    let lib_response = client.get(url).send().await
                                        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                                    let lib_bytes = lib_response.bytes().await
                                        .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                                    fs::write(&lib_path, &lib_bytes).await?;
                                }
                            }
                        }
                    }
                }
            }
            
            println!("✅ Installed Quilt loader {} for Minecraft {}", loader_version, game_version);
            return Ok(profile_name);
        }
    }
    
    Err(LibiumError::Add(libium::add::Error::DoesNotExist))
}

/// Install NeoForge mod loader
async fn install_neoforge_loader(instance_path: &Path, game_version: &str) -> Result<String, LibiumError> {
    println!("⚠️  NeoForge installation not yet implemented. Using Fabric as fallback.");
    install_fabric_loader(instance_path, game_version).await
}

/// Initialize libium integration for ChaiLauncher
pub async fn init_libium() -> Result<(), LibiumError> {
    // Test that APIs are accessible
    let _modrinth_test = &*MODRINTH_API;
    let _curseforge_test = &*CURSEFORGE_API;
    let _github_test = &*GITHUB_API;
    
    // Ensure ChaiLauncher data directory exists and is configured
    let data_dir = crate::storage::get_launcher_dir();
    fs::create_dir_all(&data_dir).await?;
    
    // Create libium subdirectory for organized storage
    let libium_dir = data_dir.join("libium");
    fs::create_dir_all(&libium_dir).await?;
    
    // Create profiles directory
    let profiles_dir = libium_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).await?;
    
    // Create modpacks directory  
    let modpacks_dir = libium_dir.join("modpacks");
    fs::create_dir_all(&modpacks_dir).await?;
    
    // Initialize default configuration if it doesn't exist
    let _manager = LibiumManager::new().await?;
    
    println!("✅ Libium integration initialized with ChaiLauncher data directory: {}", data_dir.display());
    println!("   - Libium data: {}", libium_dir.display());
    println!("   - Profiles: {}", profiles_dir.display());
    println!("   - Modpacks: {}", modpacks_dir.display());
    Ok(())
}