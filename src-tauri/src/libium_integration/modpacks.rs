use crate::libium_integration::{LibiumManager, LibiumError, types::*};
use libium::{
    config::structs::{Modpack, ModpackIdentifier},
};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use serde_json::json;
use tokio::fs;
use reqwest::Client;
use uuid::Uuid;
use std::io::Read;

pub struct ModpackManager {
    manager: LibiumManager,
    client: Client,
}

impl ModpackManager {
    pub async fn new() -> Result<Self, LibiumError> {
        let manager = LibiumManager::new().await?;
        let client = Client::new();
        
        Ok(Self { manager, client })
    }
    
    /// Search for modpacks on supported platforms
    pub async fn search_modpacks(
        &self,
        query: &str,
        platform: &str,
        game_version: Option<String>,
        mod_loader: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModpackSearchResult>, LibiumError> {
        match platform {
            "modrinth" => {
                println!("Searching Modrinth for modpacks...");
                self.search_modrinth_modpacks(query, game_version, mod_loader, limit, offset).await
            }
            "curseforge" => {
                println!("Searching CurseForge for modpacks...");
                self.search_curseforge_modpacks(query, game_version, mod_loader, limit, offset).await
            }
            "all" => {
                println!("Searching all platforms for modpacks...");
                let mut all_results = Vec::new();
                
                // Search Modrinth first
                if let Ok(modrinth_results) = self.search_modrinth_modpacks(query, game_version.clone(), mod_loader.clone(), limit / 2, offset).await {
                    all_results.extend(modrinth_results);
                }
                
                // Then search CurseForge
                if let Ok(curseforge_results) = self.search_curseforge_modpacks(query, game_version, mod_loader, limit / 2, offset).await {
                    all_results.extend(curseforge_results);
                }
                
                // Limit final results
                all_results.truncate(limit as usize);
                Ok(all_results)
            }
            _ => Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
        }
    }
    
    /// Search Modrinth for modpacks
    async fn search_modrinth_modpacks(
        &self,
        query: &str,
        game_version: Option<String>,
        mod_loader: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModpackSearchResult>, LibiumError> {
        let mut url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit={}&offset={}&project_type=modpack",
            urlencoding::encode(query),
            limit,
            offset
        );
        
        // Add facets for filtering
        let mut facets = Vec::new();
        
        if let Some(version) = game_version {
            facets.push(format!("\"versions:{}\"", version));
        }
        
        if let Some(loader) = mod_loader {
            facets.push(format!("\"categories:{}\"", loader));
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
            
        // Use the same ModrinthSearchResponse from mod.rs
        let search_response: crate::libium_integration::ModrinthSearchResponse = response
            .json()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        let results = search_response.hits.into_iter().map(|hit| {
            ModpackSearchResult {
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
                categories: hit.categories,
            }
        }).collect();
        
        Ok(results)
    }
    
    /// Search CurseForge for modpacks
    async fn search_curseforge_modpacks(
        &self,
        query: &str,
        game_version: Option<String>,
        mod_loader: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModpackSearchResult>, LibiumError> {
        let mut url = if query.is_empty() {
            // No search filter, omit searchFilter param
            format!(
                "https://api.curseforge.com/v1/mods/search?gameId=432&classId=4471&page_size={}&index={}",
                limit,
                offset
            )
        } else {
            // Include searchFilter param
            format!(
                "https://api.curseforge.com/v1/mods/search?gameId=432&classId=4471&searchFilter={}&page_size={}&index={}",
                urlencoding::encode(query),
                limit,
                offset
            )
        };

        println!("CurseForge search URL: {}", url);

        if let Some(version) = game_version {
            url.push_str(&format!("&gameVersion={}", version));
        }

        println!("CurseForge search URL: {}", url);

        if let Some(loader) = mod_loader {
            let loader_id = match loader.as_str() {
                "fabric" => 4,
                "forge" => 1,
                "quilt" => 5,
                "neoforge" => 6,
                _ => 4, // fallback to Fabric
            };
            url.push_str(&format!("&modLoaderType={}", loader_id));
        }

        println!("CurseForge search URL: {}", url);

        // Get CurseForge API key from environment or use default
        let api_key = std::env::var("CURSEFORGE_API_KEY")
            .unwrap_or_else(|_| "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.".to_string());

        println!("CurseForge API key: {}", api_key);

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("x-api-key", api_key)
            .header("User-Agent", "ChaiLauncher/1.0")
            .send()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        println!("CurseForge search response: {:?}", response);

        // Use the same CurseForgeSearchResponse from mod.rs
        let search_response = response
            .json()
            .await;

        println!("CurseForge search response (raw): {:?}", search_response);

        let search_response: crate::libium_integration::CurseForgeSearchResponse = search_response.map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        println!("CurseForge search response (parsed): {:?}", search_response);

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
            
            let categories = hit.categories.iter()
                .map(|cat| cat.name.clone())
                .collect();
            
            ModpackSearchResult {
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
                categories,
            }
        }).collect();

        println!("CurseForge search results: {:?}", results);
        
        Ok(results)
    }
    
    /// Get versions for a specific modpack
    pub async fn get_modpack_versions(
        &self,
        modpack_id: &str,
        platform: &str,
    ) -> Result<Vec<ModpackVersion>, LibiumError> {
        let mut versions = Vec::new();
        
        match platform {
            "modrinth" => {
                if let Ok(project_versions) = libium::MODRINTH_API.version_list(modpack_id).await {
                    for version in project_versions {
                        let files = version.files.iter().map(|file| ModpackFile {
                            filename: file.filename.clone(),
                            url: file.url.to_string(),
                            size: file.size as u64,
                            hashes: {
                                let mut hash_map = std::collections::HashMap::new();
                                hash_map.insert("sha1".to_string(), file.hashes.sha1.clone());
                                if !file.hashes.sha512.is_empty() {
                                    hash_map.insert("sha512".to_string(), file.hashes.sha512.clone());
                                }
                                hash_map
                            },
                            primary: file.primary,
                            file_type: if file.filename.ends_with(".mrpack") {
                                "mrpack".to_string()
                            } else {
                                "unknown".to_string()
                            },
                        }).collect();
                        
                        let dependencies = version.dependencies.iter().map(|dep| ModpackDependency {
                            project_id: dep.project_id.clone(),
                            version_id: dep.version_id.clone(),
                            file_name: None,
                            dependency_type: format!("{:?}", dep.dependency_type),
                        }).collect();
                        
                        versions.push(ModpackVersion {
                            id: version.id,
                            name: version.name,
                            version_number: version.version_number,
                            game_versions: version.game_versions,
                            loaders: version.loaders,
                            version_type: format!("{:?}", version.version_type),
                            featured: version.featured,
                            date_published: version.date_published.to_rfc3339(),
                            downloads: version.downloads as u64,
                            changelog: version.changelog,
                            files,
                            dependencies,
                        });
                    }
                }
            }
            "curseforge" => {
                if let Ok(project) = libium::CURSEFORGE_API.get_mod(modpack_id.parse().unwrap_or(0)).await {
                    for file in project.latest_files {
                        versions.push(ModpackVersion {
                            id: file.id.to_string(),
                            name: file.display_name,
                            version_number: file.file_name.clone(),
                            game_versions: file.game_versions.clone(),
                            loaders: file.game_versions.iter().filter_map(|v| {
                                if v.contains("forge") { Some("forge".to_string()) }
                                else if v.contains("fabric") { Some("fabric".to_string()) }
                                else { None }
                            }).collect(),
                            version_type: format!("{:?}", file.release_type),
                            featured: false,
                            date_published: file.file_date.to_rfc3339(),
                            downloads: file.download_count as u64,
                            changelog: None,
                            files: vec![ModpackFile {
                                filename: file.file_name,
                                url: file.download_url.map(|u| u.to_string()).unwrap_or_else(|| "".to_string()),
                                size: file.file_length as u64,
                                hashes: std::collections::HashMap::new(),
                                primary: true,
                                file_type: "zip".to_string(),
                            }],
                            dependencies: vec![], // CurseForge dependencies would need separate API calls
                        });
                    }
                }
            }
            _ => return Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
        }
        
        Ok(versions)
    }
    
    /// Install a modpack using libium
    pub async fn install_modpack(
        &mut self,
        request: &InstallModpackRequest,
        _app_handle: AppHandle,
        progress_callback: impl Fn(u64, u64, String) + Send + Clone + 'static,
    ) -> Result<String, LibiumError> {
        // Validate required fields
        if request.instance_name.is_empty() {
            return Err(LibiumError::Add(libium::add::Error::DoesNotExist));
        }
        if request.modpack_id.is_empty() {
            return Err(LibiumError::Add(libium::add::Error::DoesNotExist));
        }
        if request.version_id.is_empty() {
            return Err(LibiumError::Add(libium::add::Error::DoesNotExist));
        }
        if request.platform.is_empty() {
            return Err(LibiumError::Add(libium::add::Error::DoesNotExist));
        }
        progress_callback(0, 100, "Starting modpack installation".to_string());
        
        // Create instance directory
        let instances_dir = crate::storage::get_launcher_dir().join("instances");
        let instance_path = instances_dir.join(&request.instance_name);
        
        fs::create_dir_all(&instance_path).await?;
        progress_callback(10, 100, "Created instance directory".to_string());
        
        // Create modpack configuration
        let identifier = match request.platform.as_str() {
            "modrinth" => ModpackIdentifier::ModrinthModpack(request.modpack_id.clone()),
            "curseforge" => ModpackIdentifier::CurseForgeModpack(
                request.modpack_id.parse().map_err(|_| LibiumError::Add(libium::add::Error::DoesNotExist))?
            ),
            _ => return Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
        };
        
        let modpack = Modpack {
            name: request.instance_name.clone(),
            output_dir: instance_path.clone(),
            install_overrides: request.install_overrides,
            identifier: identifier.clone(),
        };
        
        progress_callback(20, 100, "Configured modpack settings".to_string());
        
        // Add modpack to config
        self.manager.get_config_mut().modpacks.push(modpack);
        
        // Install modpack based on platform and get mod count
        let mod_count = match request.platform.as_str() {
            "modrinth" => {
                self.install_modrinth_modpack(
                    &request.modpack_id,
                    &request.version_id,
                    &instance_path,
                    request.install_overrides,
                    &progress_callback,
                ).await?
            }
            "curseforge" => {
                self.install_curseforge_modpack(
                    &request.modpack_id,
                    &request.version_id,
                    &instance_path,
                    request.install_overrides,
                    &progress_callback,
                ).await?
            }
            _ => return Err(LibiumError::Add(libium::add::Error::DoesNotExist)),
        };
        
        progress_callback(90, 100, "Registering instance".to_string());
        
        // Get modpack details for proper instance metadata
        let (game_version, mod_loader) = self.get_modpack_details(&request.modpack_id, &request.version_id, &request.platform).await
            .unwrap_or_else(|_| ("1.20.1".to_string(), "fabric".to_string()));
        
        // Use the mod loader profile name as the version for the instance
        let instance_version = if let Ok(profile_name) = self.get_modpack_profile_name(&request.modpack_id, &request.version_id, &request.platform).await {
            profile_name
        } else {
            game_version.clone()
        };
        
        // Register instance in ChaiLauncher storage
        let instance_metadata = crate::storage::InstanceMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.instance_name.clone(),
            version: instance_version,
            game_dir: instance_path,
            java_path: None,
            jvm_args: None,
            last_played: None,
            total_play_time: 0,
            icon: None,
            is_modded: true,
            mods_count: mod_count, // Use actual mod count from installation
            modpack: Some(request.instance_name.clone()),
            modpack_version: Some(request.version_id.clone()),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_mb: None,
            description: Some(format!("Modpack instance with {} for Minecraft {}", mod_loader, game_version)),
            tags: vec!["modpack".to_string(), request.platform.clone(), mod_loader.clone()],
            resolved_java_version: None,
            java_analysis_date: None,
        };
        
        let mut storage_manager = crate::storage::StorageManager::new().await
            .map_err(|_| LibiumError::InvalidDataDirectory)?;
        
        storage_manager.add_instance(instance_metadata).await
            .map_err(|_| LibiumError::InvalidDataDirectory)?;
        
        progress_callback(100, 100, "Modpack installation completed".to_string());
        
        Ok(request.instance_name.clone())
    }
    
    /// Install a Modrinth modpack
    async fn install_modrinth_modpack(
        &mut self,
        modpack_id: &str,
        version_id: &str,
        instance_path: &Path,
        install_overrides: bool,
        progress_callback: &impl Fn(u64, u64, String),
    ) -> Result<u32, LibiumError> {
        progress_callback(10, 100, "Fetching modpack information".to_string());
        
        // Get modpack version information from Modrinth API
        let client = reqwest::Client::new();
        let version_url = format!("https://api.modrinth.com/v2/version/{}", version_id);
        let version_response = client.get(&version_url)
            .header("User-Agent", "ChaiLauncher/1.0")
            .send()
            .await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let version_data: serde_json::Value = version_response.json().await
            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        // Extract mod loader and game version from modpack
        let game_versions = version_data.get("game_versions")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("1.20.1");
        
        let loaders = version_data.get("loaders")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("fabric");
        
        progress_callback(20, 100, format!("Installing {} mod loader for {}", loaders, game_versions));
        
        // Install the mod loader first and get the profile name
        let mod_loader_profile = crate::libium_integration::install_mod_loader(instance_path, game_versions, loaders).await?;
        
        progress_callback(30, 100, "Downloading Modrinth modpack".to_string());
        
        // Download the modpack file
        if let Some(files) = version_data.get("files").and_then(|f| f.as_array()) {
            for file in files {
                if let (Some(url), Some(filename)) = (
                    file.get("url").and_then(|u| u.as_str()),
                    file.get("filename").and_then(|f| f.as_str())
                ) {
                    if filename.ends_with(".mrpack") {
                        progress_callback(40, 100, format!("Downloading {}", filename));
                        
                        // Download the modpack file
                        let file_response = client.get(url).send().await
                            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                        let file_bytes = file_response.bytes().await
                            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                        
                        // Save and extract modpack
                        let modpack_file = instance_path.join(filename);
                        fs::write(&modpack_file, &file_bytes).await?;
                        
                        progress_callback(60, 100, "Extracting modpack contents".to_string());
                        
                        // Extract the modpack using zip
                        let file = std::fs::File::open(&modpack_file)
                            .map_err(|e| LibiumError::Io(e))?;
                        let mut archive = zip::ZipArchive::new(file)
                            .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                        
                        // Create mods directory
                        let mods_dir = instance_path.join("mods");
                        fs::create_dir_all(&mods_dir).await?;
                        
                        let mut mods_extracted = 0;
                        let mut overrides_extracted = 0;
                        
                        // Extract mods and overrides
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i)
                                .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                            
                            let outpath = instance_path.join(file.mangled_name());
                            
                            if file.name().starts_with("mods/") {
                                if let Some(parent) = outpath.parent() {
                                    std::fs::create_dir_all(parent)
                                        .map_err(|e| LibiumError::Io(e))?;
                                }
                                
                                let mut outfile = std::fs::File::create(&outpath)
                                    .map_err(|e| LibiumError::Io(e))?;
                                std::io::copy(&mut file, &mut outfile)
                                    .map_err(|e| LibiumError::Io(e))?;
                                
                                mods_extracted += 1;
                                println!("📦 Extracted mod: {}", file.name());
                            } else if install_overrides && file.name().starts_with("overrides/") {
                                // Strip "overrides/" prefix when extracting
                                let file_name = file.name().to_string();
                                let relative_path = file_name.strip_prefix("overrides/").unwrap_or(&file_name);
                                let override_path = instance_path.join(relative_path);
                                
                                if let Some(parent) = override_path.parent() {
                                    std::fs::create_dir_all(parent)
                                        .map_err(|e| LibiumError::Io(e))?;
                                }
                                
                                let mut outfile = std::fs::File::create(&override_path)
                                    .map_err(|e| LibiumError::Io(e))?;
                                std::io::copy(&mut file, &mut outfile)
                                    .map_err(|e| LibiumError::Io(e))?;
                                
                                overrides_extracted += 1;
                                println!("📁 Extracted override: {}", relative_path);
                            }
                        }
                        
                        println!("✅ Extracted {} mods and {} overrides from modpack", mods_extracted, overrides_extracted);
                        
                        // Clean up the downloaded modpack file
                        let _ = fs::remove_file(&modpack_file).await;
                        
                        progress_callback(80, 100, "Finalizing installation".to_string());
                        
                        return Ok(mods_extracted);
                    }
                }
            }
        }
        
        progress_callback(80, 100, "Finalizing installation".to_string());
        
        Ok(0) // No mods found
    }
    
    /// Install a CurseForge modpack
    async fn install_curseforge_modpack(
        &mut self,
        modpack_id: &str,
        _version_id: &str,
        instance_path: &Path,
        install_overrides: bool,
        progress_callback: &impl Fn(u64, u64, String),
    ) -> Result<u32, LibiumError> {
        progress_callback(10, 100, "Fetching CurseForge modpack information".to_string());
        
        // For CurseForge, we need to implement proper API calls
        // This is more complex due to CurseForge's API requirements
        
        // For now, install Fabric as default mod loader for CurseForge modpacks
        progress_callback(20, 100, "Installing Fabric mod loader (default for CurseForge)".to_string());
        
        let _mod_loader_profile = crate::libium_integration::install_mod_loader(instance_path, "1.20.1", "fabric").await?;
        
        progress_callback(30, 100, "Setting up CurseForge modpack structure".to_string());
        
        // Create necessary directories
        let mods_dir = instance_path.join("mods");
        let overrides_dir = instance_path.join("overrides");
        
        fs::create_dir_all(&mods_dir).await?;
        if install_overrides {
            fs::create_dir_all(&overrides_dir).await?;
        }
        
        progress_callback(80, 100, "CurseForge modpack setup completed".to_string());
        
        println!("⚠️  CurseForge modpack installation is partially implemented.");
        println!("   The mod loader has been installed and directory structure created.");
        println!("   Full CurseForge modpack support requires additional API integration.");
        
        Ok(0) // No mods installed for CurseForge yet
    }
    
    /// Get modpack profile name based on the mod loader that was installed
    async fn get_modpack_profile_name(
        &self,
        modpack_id: &str,
        version_id: &str,
        platform: &str,
    ) -> Result<String, LibiumError> {
        let (game_version, mod_loader) = self.get_modpack_details(modpack_id, version_id, platform).await?;
        
        // This should match the profile name created by install_mod_loader
        match mod_loader.as_str() {
            "fabric" => {
                // Get latest Fabric loader version to match the profile name
                let versions_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", game_version);
                let response = self.client.get(&versions_url)
                    .header("User-Agent", "ChaiLauncher/1.0")
                    .send()
                    .await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
                let loader_versions: serde_json::Value = response.json().await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                
                if let Some(latest_loader) = loader_versions.as_array().and_then(|arr| arr.first()) {
                    if let Some(loader_version) = latest_loader.get("loader").and_then(|l| l.get("version")).and_then(|v| v.as_str()) {
                        return Ok(format!("fabric-loader-{}-{}", loader_version, game_version));
                    }
                }
            }
            "quilt" => {
                // Get latest Quilt loader version to match the profile name
                let versions_url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", game_version);
                let response = self.client.get(&versions_url)
                    .header("User-Agent", "ChaiLauncher/1.0")
                    .send()
                    .await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
                let loader_versions: serde_json::Value = response.json().await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                
                if let Some(latest_loader) = loader_versions.as_array().and_then(|arr| arr.first()) {
                    if let Some(loader_version) = latest_loader.get("loader").and_then(|l| l.get("version")).and_then(|v| v.as_str()) {
                        return Ok(format!("quilt-loader-{}-{}", loader_version, game_version));
                    }
                }
            }
            "forge" => {
                // For Forge, use the game version since Forge isn't fully implemented
                return Ok(format!("forge-{}", game_version));
            }
            _ => {
                // Default to game version for unknown loaders
                return Ok(game_version);
            }
        }
        
        // Fallback to game version
        Ok(game_version)
    }
    
    /// Get modpack details (game version and mod loader) from platform APIs
    async fn get_modpack_details(
        &self,
        modpack_id: &str,
        version_id: &str,
        platform: &str,
    ) -> Result<(String, String), LibiumError> {
        match platform {
            "modrinth" => {
                let version_url = format!("https://api.modrinth.com/v2/version/{}", version_id);
                let version_response = self.client.get(&version_url)
                    .header("User-Agent", "ChaiLauncher/1.0")
                    .send()
                    .await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
                let version_data: serde_json::Value = version_response.json().await
                    .map_err(|e| LibiumError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                
                let game_version = version_data.get("game_versions")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.20.1")
                    .to_string();
                
                let mod_loader = version_data.get("loaders")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("fabric")
                    .to_string();
                
                Ok((game_version, mod_loader))
            }
            "curseforge" => {
                // For CurseForge, we'd need to fetch project details
                // For now, return default values
                Ok(("1.20.1".to_string(), "fabric".to_string()))
            }
            _ => Ok(("1.20.1".to_string(), "fabric".to_string())),
        }
    }
    
    /// Update a modpack to a newer version
    pub async fn update_modpack(
        &mut self,
        instance_name: &str,
        new_version_id: &str,
        progress_callback: impl Fn(u64, u64, String),
    ) -> Result<(), LibiumError> {
        // Find and clone the modpack info to avoid borrowing issues
        let (output_dir, install_overrides, identifier) = {
            let modpack = self.manager.get_config_mut().modpacks.iter()
                .find(|m| m.name == instance_name)
                .ok_or_else(|| LibiumError::ProfileNotFound(instance_name.to_string()))?;
            (modpack.output_dir.clone(), modpack.install_overrides, modpack.identifier.clone())
        };
        
        progress_callback(10, 100, "Backing up current modpack".to_string());
        
        // Backup current installation
        let backup_dir = output_dir.with_extension("backup");
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).await?;
        }
        fs::rename(&output_dir, &backup_dir).await?;
        
        progress_callback(30, 100, "Downloading new version".to_string());
        
        // Reinstall with new version
        fs::create_dir_all(&output_dir).await?;
        
        match &identifier {
            ModpackIdentifier::ModrinthModpack(id) => {
                self.install_modrinth_modpack(
                    id,
                    new_version_id,
                    &output_dir,
                    install_overrides,
                    &progress_callback,
                ).await?;
            }
            ModpackIdentifier::CurseForgeModpack(id) => {
                self.install_curseforge_modpack(
                    &id.to_string(),
                    new_version_id,
                    &output_dir,
                    install_overrides,
                    &progress_callback,
                ).await?;
            }
        }
        
        progress_callback(90, 100, "Cleaning up".to_string());
        
        // Remove backup if successful
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).await?;
        }
        
        progress_callback(100, 100, "Modpack updated successfully".to_string());
        
        Ok(())
    }
}